use std::path::Path;
use regex::Regex;
use std::env;

use crate::model::*;
use crate::sbt::*;

mod model;
mod export;
mod sbt;

fn main() {
  let cd = env::current_dir().expect("Could not find current dir"); // If this fails we have other issues
  let re = Regex::new(SBT_VERSION_REGEX).expect("Could not create regex"); // Fail as we should write correct regexes.
  let current_directory = cd.to_string_lossy();

  let project_name_type =
    cd
      .file_name()
      .map(|f| ProjectName::ProjectDir(f.to_string_lossy().to_string()))
      .unwrap_or_else(ProjectName::Random);

  // TODO: Move this to sbt.rs
  if !Path::new(BUILD_SBT).exists() {
    println!("Could not find {}. Please run this in an SBT project directory", BUILD_SBT)
  } else {
    match verify_sbt_version(re) {
      SBTVersion::UnsupportedVersion(sbt_version) => println!("Required SBT version >= {}. Your version: {}", MIN_SBT_VERSION_STRING, sbt_version),
      SBTVersion::UnknownVersionString(sbt_version) => println!("Unknown SBT version string: {}", sbt_version),
      SBTVersion::NotFound => println!("Could not find {}. Please run this in an SBT project directory", SBT_BUILD_PROPERTIES),
      SBTVersion::Valid => {
        match run_sbt() {
          SBTExecution::CouldNotRun(error) => println!("Could not run sbt: {}", error),
          SBTExecution::CouldNotDecodeOutput(error) => println!("Invalid UTF8 output from sbt: {}", error),
          SBTExecution::UnrecognisedOutputStructure(error) => println!("Unrecognised output format from sbt: {}", error),
          SBTExecution::SuccessfulExecution(project_type) => handle_project_type(&project_name_type, &current_directory, &project_type)

        }
      }
    }
  }
}
