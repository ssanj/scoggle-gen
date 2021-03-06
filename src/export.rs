use crate::model::*;
use std::io::{self, Read};
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

pub fn write_new_project_file(project_file_content: &[u8], sublime_project_file: &str) -> io::Result<()> {
  let open_result =
    fs::OpenOptions::new()
      .create_new(true)
      .write(true)
      .open(sublime_project_file);

  open_result
    .and_then(|mut file| file.write_all(project_file_content))
}

pub fn overwrite_project_file(project_file_content: &[u8], sublime_project_file: &str) -> io::Result<()> {
  let open_result =
    fs::OpenOptions::new()
      .truncate(true)
      .write(true)
      .open(sublime_project_file);

  open_result
    .and_then(|mut file| file.write_all(project_file_content))
}


pub fn write_sublime_project_file(st_project_json: &str, sublime_project_file: &str) {
  let project_file_content = st_project_json.to_string();
  let project_file_content_bytes = project_file_content.as_bytes();
  let project_file_written = write_new_project_file(project_file_content_bytes, sublime_project_file);

  match project_file_written {
    Ok(_) => print_success(format!("Successfully generated {}", sublime_project_file)),
    Err(error) => {
      if error.kind() == io::ErrorKind::AlreadyExists {
        match read_overwrite_response(&format!("{} already exists. Overwrite Y/N ?", sublime_project_file)) {
          Confirmation::Yes => {
            println!("Overwriting {}", sublime_project_file);
            match overwrite_project_file(project_file_content_bytes, sublime_project_file) {
              Ok(_) => print_success(format!("Successfully generated {}", sublime_project_file)),
              Err(e) => {
                print_error(format!("Could not overwrite {} because of error: {}", sublime_project_file, e));
                println!("Writing content to stdout:");
                write_project_to_stdout(&project_file_content)
              }
            }
          },
          Confirmation::No => {
            println!("Writing content to stdout:");
            write_project_to_stdout(&project_file_content)
          }
        }
      } else {
        print_error(format!("Could not write {} due to: {}. Writing content to stdout:", sublime_project_file, error))
      }
    }
  }
}

fn write_project_to_stdout(project_file_content: &str) {
    println!("```");
    println!("{}", project_file_content);
    println!("```")
}

fn read_overwrite_response(question: &str) -> Confirmation {
  let mut reader = io::stdin();
  let mut buffer: [u8;1] = [0];
  println!("{}", question);
  match reader.read_exact(&mut buffer) {
    Ok(_) => {
      let response = buffer[0] as char;
      match response {
        'Y' | 'y' => Confirmation::Yes,
        _ => Confirmation::No
      }
    },
    Err(_) => Confirmation::No
  }
}
