use std::process::Command;
use std::io::{self};
use std::str::from_utf8;

fn main() -> io::Result<()> {
    // check for build.sbt
    // check sbt version
    println!("Running SBT...");

    let output = Command::new("sbt")
                    .arg("print baseDirectory")
                    .arg("--error")
                    .output()?;

    let output_str = from_utf8(&output.stdout).unwrap();
    let lines:Vec<_> = output_str.lines().collect();

    //return as a Result of Vec<str>
    if lines.len() == 1 {
        //single module
        println!("{}", &lines[0]);
    } else if lines.len() % 2 == 0 {
        //multimodule, 2 lines per module
        // line1: moduleName / baseDirectory <- we can ignore this
        // line2: <actual module path> <- we need this
        lines
            .iter()
            .enumerate()
            .filter(|(i, _)| (i+1) % 2 == 0) // only get the second line
            .for_each(|(_, v)| println!("{}", v));
    } else {
        //big fat error!
        println!("invalid output: {:?}", &lines);
    }

    //map each path to src/main/scala + src/test/scala
    //use Serde to down out to ST project structure as JSON.

    Ok(())
}
