# README

A tool that generates a Sublime Text project file that helps you get started using [Scoggle](https://packagecontrol.io/packages/Scoggle). While Scoggle-Gen may not find every single source and test root you have, it will get you setup pretty quick and you can then hack away at it to your hearts content.

If you have a bizzaro setup of source and test sources, then you only have ourself to blame and can create your own project file :P

## Usage

run scoggle-gen in the root of your SBT project folder.

It requires a minimum of SBT 1.4.5 to be used on the project for it to generate your project file.

## Installation

### Build from source

- Git clone this repo
- Run `cargo build --release`
- Copy the executable from the `target/release/scoggle-gen` into a folder on your path

### From GH

- TODO
