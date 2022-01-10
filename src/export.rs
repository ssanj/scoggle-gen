use crate::model::*;
use std::io;
use std::io::Write;
use std::fs;
use crate::term::*;

pub fn build_sublime_project(prod_sources: Vec<ProdSource>, test_sources: Vec<TestSource>) -> SublimeProject {
  let po = PathObject { path: ".".to_owned() };

  let scoggle  =
    ScoggleObject {
      production_srcs: prod_sources.iter().map(|ps| ps.0.to_owned()).collect(),
      test_srcs: test_sources.iter().map(|ts| ts.0.to_owned()).collect(),
      test_suffixes: vec!["Spec.scala".to_string(), "Suite.scala".to_string(), "Test.scala".to_string()],
    };

  let settings_object =
    SettingsObject {
      scoggle
    };

  SublimeProject {
    folders: vec![po],
    settings: settings_object
  }

}

pub fn write_project_file(project_file_content: &[u8], sublime_project_file: &str) -> io::Result<()> {
  let open_result =
    fs::OpenOptions::new()
      .create_new(true)
      .write(true)
      .open(sublime_project_file);

  open_result
    .and_then(|mut file| file.write_all(project_file_content))
}


pub fn write_sublime_project_file(st_project_json: &str, sublime_project_file: &str) {
  let project_file_content = st_project_json.to_string();
  let project_file_content_bytes = project_file_content.as_bytes();
  let project_file_written = write_project_file(project_file_content_bytes, sublime_project_file); //{

  match project_file_written {
    Ok(_) => print_success(format!("Successfully generated {}", sublime_project_file)),
    Err(error) => {
      if error.kind() == io::ErrorKind::AlreadyExists {
        print_error(format!("Output file {} already exists. Writing content to stdout:", sublime_project_file))
      } else {
        print_error(format!("Could not write {} due to: {}. Writing content to stdout:", sublime_project_file, error))
      }

      println!("```");
      println!("{}", project_file_content);
      println!("```")
    }
  }
}
