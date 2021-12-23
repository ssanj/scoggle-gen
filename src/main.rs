use std::{process::Command};
use std::io;
use std::str::from_utf8;
use std::path::Path;
use regex::{Regex, Captures};
use std::fs;

use std::env;
use uuid::Uuid;

use crate::model::*;
use crate::export::*;

mod model;
mod export;

const SBT_BUILD_PROPERTIES: &str = "project/build.properties";
const BUILD_SBT: &str = "build.sbt";
const MIN_SBT_VERSION: u16 = 145;
const MIN_SBT_VERSION_STRING: &str = "1.4.5";
const SCALA_PROD_PATH: &str = "/src/main/scala";
const SCALA_TEST_PATH: &str = "/src/test/scala";
const SBT_VERSION_REGEX: &str = r"sbt.version\s*=\s*(.+)";

fn main() {
  let cd = env::current_dir().expect("Could not find current dir"); // If this fails we have other issues
  let re = Regex::new(SBT_VERSION_REGEX).expect("Could not create regex"); // Fail as we should write correct regexes.
  let current_directory = cd.to_string_lossy();

  let project_name_type =
    cd
      .file_name()
      .map(|f| ProjectName::ProjectDir(f.to_string_lossy().to_string()))
      .unwrap_or_else(|| ProjectName::Random());

  if !Path::new(BUILD_SBT).exists() {
    println!("Could not find {}. Please run this in an SBT project directory", BUILD_SBT)
  } else {
    match verify_sbt_version(re) {
      SBTVersion::UnsupportedSBTVersion(sbt_version) => println!("Required SBT version >= {}. Your version: {}", MIN_SBT_VERSION_STRING, sbt_version),
      SBTVersion::UnknownVersionString(sbt_version) => println!("Unknown SBT version string: {}", sbt_version),
      SBTVersion::NotFound => println!("Could not find {}. Please run this in an SBT project directory", SBT_BUILD_PROPERTIES),
      SBTVersion::Valid => {
        match run_sbt() {
          SBTExecution::CouldNotRun(error) => println!("Could not run sbt: {}", error),
          SBTExecution::CouldNotDecodeOutput(error) => println!("Invalid UTF8 output from sbt: {}", error),
          SBTExecution::UnrecognisedOutputStructure(error) => println!("Unrecognised output format from sbt: {}", error),
          SBTExecution::SuccessfulExecution(project_type) => handle_project_type(project_name_type, &current_directory, project_type)

        }
      }
    }
  }
}

fn default_project_name() -> String  {
  format!("scoggle-gen-{}",Uuid::new_v4())
}

fn get_project_name(project_name_type: ProjectName) -> String {
  match project_name_type {
    ProjectName::ProjectDir(pn) => pn,
    ProjectName::Random() => {
      let random = default_project_name();
      eprintln!("Could not retrieve project name. Using generated name: {}", random);
      random
    }
  }
}

fn handle_project_type(project_name_type: ProjectName, current_directory: &str, project_type: ProjectType) {
  let project_name = get_project_name(project_name_type);
  let sublime_project_file = format!("{}.sublime-project", project_name);

  let ProjectType(projects) = project_type.clone();
  let pairs: Vec<(ProdSource, TestSource)> =
    projects
      .iter()
      .map(|p| {
          let relative_path = p.replace(current_directory, "");
          (ProdSource(format!("{}{}", relative_path, SCALA_PROD_PATH)),  TestSource(format!("{}{}", relative_path, SCALA_TEST_PATH)))
      })
      .collect();

  // TODO: Simplify, with a fold
  let prod_sources: Vec<&ProdSource> = pairs.iter().map(|(p,_)| p).collect();
  let test_sources: Vec<&TestSource> = pairs.iter().map(|(_,t)| t).collect();
  let sublime_project = build_sublime_project(prod_sources, test_sources);

  match serde_json::to_string_pretty(&sublime_project) {
    Ok(st_project_json) => write_sublime_project_file(&st_project_json, &sublime_project_file),
    Err(error) => eprintln!("Could not convert Sublime Text Project model to JSON: {}", error)
  }
}


fn run_sbt() -> SBTExecution {
    println!("Running SBT, this may take a while 🙄");

    match Command::new("sbt")
        .arg("set offline := true; print baseDirectory")
        .arg("--error")
        .output() {
          Ok(output) => {
            match from_utf8(&output.stdout) {
              Ok(output_str) => get_base_directories(output_str),
              Err(error) => SBTExecution::CouldNotDecodeOutput(error.to_string())
            }
          },
          Err(error) => SBTExecution::CouldNotRun(error.to_string())
        }
}

fn get_base_directories(output_str: &str) -> SBTExecution {
  let lines:Vec<_> = output_str.lines().collect();

  if lines.len() == 1 { //Single module
    SBTExecution::SuccessfulExecution(ProjectType(vec![lines[0].trim().to_string()]))
  } else if lines.len() % 2 == 0 {
    // Multimodule, 2 lines per module
    // line1: moduleName / baseDirectory <- we can ignore this
    // line2: <actual module path> <- we need this
    SBTExecution::SuccessfulExecution(ProjectType(
      lines
        .iter()
        .enumerate()
        .filter(|(i, _)| (i+1) % 2 == 0) // only get the second line
        .map(|(_, v)| format!("{}", v.trim()))
        .collect()
    ))
  } else {
    SBTExecution::UnrecognisedOutputStructure(format!("{:?}", &lines))
  }
}

fn verify_sbt_version(re: Regex) -> SBTVersion {
  match fs::read_to_string(SBT_BUILD_PROPERTIES) {
    Ok(version) => {
      let caps: Option<Captures> = re.captures(&version);
      match caps.and_then(|group| group.get(1)).map(|m| m.as_str() ) {
        Some(sbt_version) => {
          let sbt_version_no_str = sbt_version.split(".").collect::<Vec<&str>>().join("");

          //TODO: extract function - is_valid_sbt_version
          match sbt_version_no_str.parse::<u16>() {
            Ok(sbt_version_no) => {
              if sbt_version_no < MIN_SBT_VERSION {
                SBTVersion::UnsupportedSBTVersion(sbt_version.to_owned())
              } else {
                SBTVersion::Valid
              }
            },
            Err(_) => SBTVersion::UnknownVersionString(sbt_version.to_owned())
          }
        },
        None => SBTVersion::UnknownVersionString(version.to_owned())

      }
    },
    Err(_) => SBTVersion::NotFound
  }
}
