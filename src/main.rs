// use std::process::Command;
use std::io;
// use std::str::from_utf8;
// use std::path::Path;

use crate::model::{PathObject, ScoggleObject, SettingsObject, SublimeProject};
// use serde_json::Result;

mod model;

fn main() -> io::Result<()> {
    let po = PathObject { path: ".".to_owned() };

    let scoggle  =
        ScoggleObject {
            production_srcs: vec!["one".to_string(), "two".to_string(), "three".to_string()],
            test_srcs: vec!["one-test".to_string(), "two-test".to_string(), "three-test".to_string()],
            test_suffixes: vec!["Spec".to_string(), "Suite".to_string()],
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

    println!("json:\n{}", serde_json::to_string(&sublime_project).unwrap());

    Ok(())

    // if !Path::new("build.sbt").exists() {
    //     println!("Could not find build.sbt. Please run this in an SBT project directory");
    //     Err(to_io_error("not an SBT project directory"))
    // } else if !Path::new("project/build.properties").exists() {
    //     println!("Could not find project/build.properties. Please run this in an SBT project directory");
    //     Err(to_io_error("unknown SBT version required"))
    // } else {
    //     println!("Running SBT...");

    //     let output = Command::new("sbt")
    //                     .arg("print baseDirectory")
    //                     .arg("--error")
    //                     .output()?;

    //     let output_str = from_utf8(&output.stdout).unwrap();
    //     let lines:Vec<_> = output_str.lines().collect();

    //     let line_result: Result<Vec<String>, String> =
    //         if lines.len() == 1 {
    //             Ok(vec!(lines[0].trim().to_string()))
    //         } else if lines.len() % 2 == 0 {
    //             //multimodule, 2 lines per module
    //             // line1: moduleName / baseDirectory <- we can ignore this
    //             // line2: <actual module path> <- we need this
    //             Ok(
    //                 lines
    //                     .iter()
    //                     .enumerate()
    //                     .filter(|(i, _)| (i+1) % 2 == 0) // only get the second line
    //                     .map(|(_, v)| format!("{}", v.trim()))
    //                     .collect()
    //             )
    //         } else {
    //             //big fat error!
    //             Err(format!("invalid output: {:?}", &lines))
    //         };

    //     //map each path to src/main/scala + src/test/scala
    //     //use Serde to down out to ST project structure as JSON.

    //     match line_result {
    //         Ok(lines) => {
    //             println!("config: {:?}", lines);
    //             Ok(())
    //         },
    //         Err(error) => {
    //             println!("Scoggle config generation failed");
    //             Err(to_io_error(&error))
    //         }
    //     }
    // }

}

// fn to_io_error(message: &str) -> io::Error {
//     io::Error::new(io::ErrorKind::Other, message.to_string())
// }
