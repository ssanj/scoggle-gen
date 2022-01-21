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

  let sublime_help_text: &str = &format!(
      "Generates a Sublime Text project file for Scoggle.\n\
       Run from the root of an SBT project. \n\
       Needs access to {} and {} \n\
       Supports SBT versions >= {} \n\
       See: https://packagecontrol.io/packages/Scoggle
      ", BUILD_SBT, SBT_BUILD_PROPERTIES, MIN_SBT_VERSION_STRING
  );

  let memory_help_text: &str =
      "Specifies JVM heap supplied to SBT in Megabytes.\n\
       Use this for large SBT projects that need more memory to run. \n\
       An example value is '2048' (2 Gigabytes) \n\
       Use this with the sublime option.";


  let app = App::new("scoggle-gen")
      .version(APPVERSION)
      .author("Sanj Sahayam")
      .about("Auto-generate Scoggle config for Sublime Text")
      .arg(
          Arg::new("sublime")
              .short('s')
              .long("sublime")
              .next_line_help(true)
              .help(sublime_help_text),
      )
      .arg(
          Arg::new("memory")
              .short('m')
              .long("mem")
              .takes_value(true)
              .next_line_help(true)
              .help(memory_help_text)
      );

  let mut app2 = app.clone();
  let matches = app.get_matches();

  if matches.is_present("sublime") {
    let sbt_mem = match matches.value_of("memory") {
      Some(mem_selected) => {
       mem_selected.parse::<u32>()
         .map( SBTMemory::CustomMemoryInMB)
         .unwrap_or(SBTMemory::DefaultMemory)
      },
      None => SBTMemory::DefaultMemory
    };

    run_program(sbt_mem)
  } else {
    app2.print_help().unwrap();
    println!()
  }
}

fn run_program(sbt_memory: SBTMemory) {
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
      match run_sbt(sbt_memory) {
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


