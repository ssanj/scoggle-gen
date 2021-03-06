# README

![](https://img.shields.io/github/v/release/ssanj/scoggle-gen?display_name=tag)

A tool that generates a Sublime Text project file that helps you get started using [Scoggle](https://packagecontrol.io/packages/Scoggle). While Scoggle-Gen may not find every single source and test root you have, it will get you setup pretty quick and you can then hack away at it to your hearts content.

If you have a bizzaro setup of source and test sources, then you only have ourself to blame and can create your own project file :P

## Usage

Run `scoggle-gen --help` for usage information:

```
scoggle-gen 0.1.8
Sanj Sahayam
Auto-generate Scoggle config for Sublime Text

USAGE:
    scoggle-gen [OPTIONS]

OPTIONS:
    -h, --help
            Print help information

    -m, --mem <memory>
            Specifies JVM heap supplied to SBT in Megabytes.
            Use this for large SBT projects that need more memory to run.
            An example value is '2048' (2 Gigabytes)
            Use this with the sublime option.

    -s, --sublime
            Generates a Sublime Text project file for Scoggle.
            Run from the root of an SBT project.
            Needs access to build.sbt and project/build.properties
            Supports SBT versions >= 1.4.5
            See: https://packagecontrol.io/packages/Scoggle

    -V, --version
            Print version information

```

To generate a Sublime Text project configuration for Scoggle, run `scoggle-gen` in the root of your SBT project folder:

```
scoggle-gen -s
```

It requires a minimum of SBT 1.4.5 to be used on the project for it to generate your project file.

## Installation

### Download

Download from [release](https://github.com/ssanj/scoggle-gen/releases) page.


### Build from source

- Git clone this repo
- Run `cargo build --release`
- Copy the executable from the `target/release/scoggle-gen` into a folder on your path


## Troubleshooting

If `scoggle-gen` takes a very long time when running SBT (over a minute), then you may have to bump up the heap size supplied to SBT with the `-m` option.
