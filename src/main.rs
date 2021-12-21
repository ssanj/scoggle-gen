use std::process::Command;
use std::io;
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

    let line_result: Result<Vec<String>, String> =
        if lines.len() == 1 {
            Ok(vec!(lines[0].trim().to_string()))
        } else if lines.len() % 2 == 0 {
            //multimodule, 2 lines per module
            // line1: moduleName / baseDirectory <- we can ignore this
            // line2: <actual module path> <- we need this
            Ok(
                lines
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| (i+1) % 2 == 0) // only get the second line
                    .map(|(_, v)| format!("{}", v.trim()))
                    .collect()
            )
        } else {
            //big fat error!
            Err(format!("invalid output: {:?}", &lines))
        };

    //map each path to src/main/scala + src/test/scala
    //use Serde to down out to ST project structure as JSON.

    match line_result {
        Ok(lines) => {
            println!("config: {:?}", lines);
            Ok(())
        },
        Err(error) => {
            println!("Scoggle config generation failed");
            Err(to_io_error(&error))
        }
    }
}

fn to_io_error(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message.to_string())
}
