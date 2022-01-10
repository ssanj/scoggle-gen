use ansi_term::Colour::{Red, Green};

pub fn print_error(message: String) {
  println!("{}{}", Red.paint("Error: "), message)
}

pub fn print_success(message: String) {
  println!("{}", Green.paint(message))
}
