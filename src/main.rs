use std::{process::Command};
use std::io;
use std::str::from_utf8;
use std::path::Path;
use regex::{Regex, Captures};
use std::fs;
use std::io::Write;
use std::env;
use uuid::Uuid;


use crate::model::*;

mod model;

const SBT_BUILD_PROPERTIES: &str = "project/build.properties";
const BUILD_SBT: &str = "build.sbt";
const MIN_SBT_VERSION: u16 = 145;
const MIN_SBT_VERSION_STRING: &str = "1.4.5";
const SCALA_PROD_PATH: &str = "/src/main/scala";
const SCALA_TEST_PATH: &str = "/src/test/scala";

fn main() {
    let cd = env::current_dir().expect("Could not find current dir");
    let current_directory = cd.to_string_lossy();

    let project_name_type =
        cd
            .file_name()
            .map(|f| ProjectName::ProjectDir(f.to_string_lossy().to_string()))
            .unwrap_or_else(|| ProjectName::Random());



    //TODO: Remove
    println!("current dir: {}", current_directory);
    // println!("working dir: {}", project_name);

    if !Path::new(BUILD_SBT).exists() {
        println!("Could not find {}. Please run this in an SBT project directory", BUILD_SBT)
    } else if !Path::new("project/build.properties").exists() {
        println!("Could not find {}. Please run this in an SBT project directory", SBT_BUILD_PROPERTIES)
    } else {
        match verify_sbt_version() {
            SBTVersion::UnsupportedSBTVersion(sbt_version) => println!("Required SBT version >= {}. Your version: {}", MIN_SBT_VERSION_STRING, sbt_version),
            SBTVersion::UnknownVersionString(sbt_version) => println!("Unknown SBT version string: {}", sbt_version),
            SBTVersion::NotFound => println!("{} was not found", SBT_BUILD_PROPERTIES),
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

fn handle_project_type(project_name_type: ProjectName, current_directory: &str, project_type: ProjectType) {
    let project_name = match project_name_type {
        ProjectName::ProjectDir(pn) => pn,
        ProjectName::Random() => {
            let random = default_project_name();
            eprintln!("Could not retrieve project name. Using generated name: {}", random);
            random
        }
    };

    let sublime_project_file = format!("{}.sublime-project", project_name);

    let ProjectType(mut projects) = project_type.clone();
    let pairs: Vec<(ProdSource, TestSource)> =
        projects
            .iter_mut().map(|p| p.replace(current_directory, ""))
            .map({ |p|
                (ProdSource(format!("{}{}", p, SCALA_PROD_PATH)),  TestSource(format!("{}{}", p, SCALA_TEST_PATH)))
        }).collect();

    let prod_sources: Vec<&ProdSource> = pairs.iter().map(|(p,_)| p).collect();
    let test_sources: Vec<&TestSource> = pairs.iter().map(|(_,t)| t).collect();
    let sublime_project = build_sublime_project(prod_sources, test_sources);

    match serde_json::to_string_pretty(&sublime_project) {
        Ok(st_project_json) => {
            //check for existing sublime-project and use random in that case
            let project_file_content = format!("{}", st_project_json);
            let project_file_written = {
                    let open_result =
                        fs::OpenOptions::new()
                            .create_new(true)
                            .write(true)
                            .open(&sublime_project_file);

                    open_result
                        .and_then(|mut file| file.write_all(&project_file_content.as_bytes()))

            };
            match project_file_written {
                Ok(_) => println!("Successfully generated {}", sublime_project_file),
                Err(error) => {
                    eprintln!("Could not write {} due to: {}. Writing content to stdout", sublime_project_file, error);
                    eprintln!("{}", project_file_content)
                }
            }
        },
        Err(error) => println!("Could not convert Sublime Text Project model to JSON: {}", error)
    }
}

fn build_sublime_project(prod_sources: Vec<&ProdSource>, test_sources: Vec<&TestSource>) -> SublimeProject {
    let po = PathObject { path: ".".to_owned() };

    let scoggle  =
        ScoggleObject {
            production_srcs: prod_sources.iter().map(|ps| ps.0.to_owned()).collect(),
            test_srcs: test_sources.iter().map(|ts| ts.0.to_owned()).collect(),
            test_suffixes: vec!["Spec.scala".to_string(), "Suite.scala".to_string(), "Test.scala".to_string()],
        };

    let settings_object =
        SettingsObject {
            scoggle: scoggle
        };

    let sublime_project =
        SublimeProject {
            folders: vec![po],
            settings: settings_object
        };

    sublime_project
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

    if lines.len() == 1 {
        SBTExecution::SuccessfulExecution(ProjectType(vec![lines[0].trim().to_string()]))
    } else if lines.len() % 2 == 0 {
        //multimodule, 2 lines per module
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
        //big fat error!
        SBTExecution::UnrecognisedOutputStructure(format!("{:?}", &lines))
    }
}

fn verify_sbt_version() -> SBTVersion {
    let re = Regex::new(r"sbt.version\s*=\s*(.+)").unwrap(); // Fail as we should write correct regexes.
    match fs::read_to_string("project/build.properties") {
        Ok(version) => {
            let caps: Option<Captures> = re.captures(&version);
            match caps.and_then(|group| group.get(1)).map(|m| m.as_str() ) {
                Some(sbt_version) => {
                    let sbt_version_no_str = sbt_version.split(".").collect::<Vec<&str>>().join("");
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
