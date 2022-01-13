use regex::Regex;
use std::env;
use clap::{App, Arg};

use crate::model::*;
use crate::sbt::*;
use crate::term::print_error;

mod model;
mod export;
mod sbt;
mod term;

fn main() {

  const APPVERSION: &str = env!("CARGO_PKG_VERSION");

  let app = App::new("scoggle-gen")
      .version(APPVERSION)
      .author("Sanj Sahayam")
      .about("Auto-generate Scoggle config for Sublime Text")
      .arg(
          Arg::new("sublime")
              .short('s')
              .long("sublime")
              .help("Generate Sublime Text Configuration")
      );

  let mut app2 = app.clone();
  let matches = app.get_matches();

  if matches.is_present("sublime") {
    run_program()
  } else {
    app2.print_help().unwrap();
    println!()
  }
}

fn run_program() {
  // If this fails we have other issues
  let cd = env::current_dir().expect("Could not find current dir");

  // Fail as we should write correct regexes.
  let re = Regex::new(SBT_VERSION_REGEX).expect("Could not create regex");

  let current_directory = cd.to_string_lossy();

  let project_name_type =
    cd
      .file_name()
      .map(|f| ProjectName::ProjectDir(f.to_string_lossy().to_string()))
      .unwrap_or_else(ProjectName::Random);

  match verify_sbt_version(re) {
    SBTVersion::UnsupportedVersion(sbt_version, SupportedSBTVersion(min_sbt_version)) =>
      print_error(format!("Required SBT version >= {}. Your version: {}", min_sbt_version, sbt_version)),
    SBTVersion::UnknownVersionString(sbt_version) =>
      print_error(format!("Unknown SBT version string: {}", sbt_version)),
    SBTVersion::BuildSBTNotFound =>
      print_error(format!("Could not find {}. Please run this in an SBT project directory", BUILD_SBT)),
    SBTVersion::BuildPropertiesNotFound =>
      print_error(
        format!("Could not find {}. Please run this in an SBT project directory", SBT_BUILD_PROPERTIES)
      ),
    SBTVersion::Valid => {
      match run_sbt() {
        SBTExecution::CouldNotRun(error) =>
          print_error(format!("Could not run sbt: {}", error)),
        SBTExecution::CouldNotDecodeOutput(error) =>
          print_error(format!("Invalid UTF8 output from sbt: {}", error)),
        SBTExecution::UnrecognisedOutputStructure(error) =>
          print_error(format!("Unrecognised output format from sbt: {}", error)),
        SBTExecution::SuccessfulExecution(project_type) =>
          handle_project_type(&project_name_type, &current_directory, &project_type)
      }
    }
  }
}


