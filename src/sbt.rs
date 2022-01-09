use crate::model::*;
use crate::export::*;
use std::time::Instant;
use std::str::from_utf8;
use std::process::Command;
use regex::{Regex, Captures};
use std::fs;
use uuid::Uuid;

pub const SBT_VERSION_REGEX: &str = r"sbt.version\s*=\s*(.+)";
pub const SBT_BUILD_PROPERTIES: &str = "project/build.properties";
pub const BUILD_SBT: &str = "build.sbt";
pub const MIN_SBT_VERSION: u16 = 145;
pub const MIN_SBT_VERSION_STRING: &str = "1.4.5";
pub const SCALA_PROD_PATH: &str = "/src/main/scala";
pub const SCALA_TEST_PATH: &str = "/src/test/scala";


pub fn run_sbt() -> SBTExecution {
  println!("Running SBT, this may take a while 🙄");
  let sbt_start = Instant::now();

  match Command::new("sbt")
    .arg("set offline := true; print baseDirectory")
    .arg("--error")
    .output() {
      Ok(output) => {
        let sbt_run_time_secs = sbt_start.elapsed().as_secs();
        match from_utf8(&output.stdout) {
          Ok(output_str) => {
            println!("SBT execution completed in {} seconds 😧", sbt_run_time_secs);
            get_base_directories(output_str)
          },
          Err(error) => SBTExecution::CouldNotDecodeOutput(error.to_string())
        }
      },
      Err(error) => SBTExecution::CouldNotRun(error.to_string())
    }
}

pub fn verify_sbt_version(re: Regex) -> SBTVersion {
  match fs::read_to_string(SBT_BUILD_PROPERTIES) {
    Ok(version) => {
      let caps: Option<Captures> = re.captures(&version);
      match caps.and_then(|group| group.get(1)).map(|m| m.as_str() ) {
        Some(sbt_version) => {
          let sbt_version_no_str = sbt_version.split('.').collect::<Vec<&str>>().join("");

          //TODO: extract function - is_valid_sbt_version
          match sbt_version_no_str.parse::<u16>() {
            Ok(sbt_version_no) => {
              if sbt_version_no < MIN_SBT_VERSION {
                SBTVersion::UnsupportedVersion(sbt_version.to_owned())
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

fn default_project_name() -> String  {
  format!("scoggle-gen-{}",Uuid::new_v4())
}

fn get_project_name(project_name_type: &ProjectName) -> String {
  match project_name_type {
    ProjectName::ProjectDir(pn) => pn.to_string(),
    ProjectName::Random() => {
      let random = default_project_name();
      eprintln!("Could not retrieve project name. Using generated name: {}", random);
      random
    }
  }
}

pub fn handle_project_type(project_name_type: &ProjectName, current_directory: &str, project_type: &ProjectType) {
  let project_name = get_project_name(project_name_type);
  let sublime_project_file = format!("{}.sublime-project", project_name);

  let (prod_sources, test_sources) = get_prod_and_test_sources(current_directory, project_type);
  let sublime_project = build_sublime_project(prod_sources, test_sources);

  match serde_json::to_string_pretty(&sublime_project) {
    Ok(st_project_json) => write_sublime_project_file(&st_project_json, &sublime_project_file),
    Err(error) => eprintln!("Could not convert Sublime Text Project model to JSON: {}", error)
  }
}

fn get_prod_and_test_sources(current_directory: &str, project_type: &ProjectType) -> (Vec<ProdSource>, Vec<TestSource>) {
  let ProjectType(projects) = project_type;
  let pairs: Vec<(ProdSource, TestSource)> =
    projects
      .iter()
      .map(|p| {
          let relative_path = p.replace(current_directory, "");
          (ProdSource(format!("{}{}", relative_path, SCALA_PROD_PATH)),  TestSource(format!("{}{}", relative_path, SCALA_TEST_PATH)))
      })
      .collect();

  let inits: (Vec<ProdSource>, Vec<TestSource>) =  (Vec::new(), Vec::new());
  // TODO: Can we solve this without lifetimes?
  pairs.iter().fold(inits, |(mut psv, mut tsv), (ps, ts)| {
    psv.push(ps.clone());
    tsv.push(ts.clone());
    (psv, tsv)
  })
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
        .map(|(_, v)| v.trim().to_string())
        .collect()
    ))
  } else {
    SBTExecution::UnrecognisedOutputStructure(format!("{:?}", &lines))
  }
}