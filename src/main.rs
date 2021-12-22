use std::{process::Command, slice::SliceIndex};
use std::io;
use std::str::from_utf8;
use std::path::Path;
use regex::{Regex, Captures};
use std::fs;

use crate::model::{PathObject, ScoggleObject, SettingsObject, SublimeProject};
// use serde_json::Result;

mod model;

const SBT_BUILD_PROPERTIES: &str = "project/build.properties";
const BUILD_SBT: &str = "build.sbt";

fn main() {
    if !Path::new(BUILD_SBT).exists() {
        println!("Could not find {}. Please run this in an SBT project directory", BUILD_SBT)
    } else if !Path::new("project/build.properties").exists() {
        println!("Could not find project/build.properties. Please run this in an SBT project directory")
    } else {
        match verify_sbt_version() {
            SBTVersion::UnsupportedSBTVersion(sbt_version) => println!("Unsupported SBT version: {}", sbt_version),
            SBTVersion::UnknownVersionString(sbt_version) => println!("Unknown SBT version string: {}", sbt_version),
            SBTVersion::NotFound => println!("{} was not found", SBT_BUILD_PROPERTIES),
            SBTVersion::Valid => {
                match run_sbt() {
                    SBTExecution::CouldNotRun(error) => println!("Could not run sbt: {}", error),
                    SBTExecution::CouldNotDecodeOutput(error) => println!("Invalid UTF8 output from sbt: {}", error),
                    SBTExecution::UnrecognisedOutputStructure(error) => println!("Unrecognised output format from sbt: {}", error),
                    SBTExecution::SingleModuleProject(base_directory) => println!("{}", base_directory),
                    SBTExecution::MultiModuleProject(base_directories) => println!("{}", base_directories.join("\n")),
                }
            }
        }
    }
}

enum SBTExecution {
    CouldNotRun(String),
    CouldNotDecodeOutput(String),
    UnrecognisedOutputStructure(String),
    SingleModuleProject(String),
    MultiModuleProject(Vec<String>),
}

fn run_sbt() -> SBTExecution {
        println!("Running SBT...");

        match Command::new("sbt")
                .arg("print baseDirectory")
                .arg("--error")
                .output() {
                    Ok(output) => {
                        match from_utf8(&output.stdout) {
                            Ok(output_str) => {
                               let lines:Vec<_> = output_str.lines().collect();

                                let line_result: SBTExecution =
                                    if lines.len() == 1 {
                                        SBTExecution::SingleModuleProject(lines[0].trim().to_string())
                                    } else if lines.len() % 2 == 0 {
                                        //multimodule, 2 lines per module
                                        // line1: moduleName / baseDirectory <- we can ignore this
                                        // line2: <actual module path> <- we need this
                                        SBTExecution::MultiModuleProject(
                                            lines
                                                .iter()
                                                .enumerate()
                                                .filter(|(i, _)| (i+1) % 2 == 0) // only get the second line
                                                .map(|(_, v)| format!("{}", v.trim()))
                                                .collect()
                                        )
                                    } else {
                                        //big fat error!
                                        SBTExecution::UnrecognisedOutputStructure(format!("{:?}", &lines))
                                    };


                                //map each path to src/main/scala + src/test/scala
                                //use Serde to down out to ST project structure as JSON.
                                line_result
                            },
                            Err(error) => SBTExecution::CouldNotDecodeOutput(error.to_string())
                        }
                    },
                    Err(error) => SBTExecution::CouldNotRun(error.to_string())
                }
}

enum SBTVersion {
    Valid,
    UnsupportedSBTVersion(String),
    UnknownVersionString(String),
    NotFound
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
                            if sbt_version_no < 150 {
                                SBTVersion::UnsupportedSBTVersion(sbt_version_no_str.to_owned())
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

fn to_io_error(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message.to_string())
}
