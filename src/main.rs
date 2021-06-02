mod ast;
mod compiler;
mod lexer;
mod parser;
mod project;
use clap::{App, Arg, SubCommand};
use parser::{Program, SourceFile};
use project::Project;

const NAME: &'static str = env!("CARGO_BIN_NAME");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &'static str = env!("CARGO_PKG_DESCRIPTION");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    // Parse args
    let matches = App::new(NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about(ABOUT)
        .subcommand(SubCommand::with_name("fmt").about("Format a plume project"))
        .subcommand(
            SubCommand::with_name("build")
                .about("Build a plume project")
                .arg(
                    Arg::with_name("target")
                        .long("target")
                        .short("t")
                        .takes_value(true),
                ),
        )
        .subcommand(SubCommand::with_name("validate").about("Build a plume project"))
        .subcommand(
            SubCommand::with_name("ast").about("View the abstract syntax tree of the project"),
        )
        .get_matches();

    // Check what subcommand was used
    if let Some(command) = matches.subcommand_name() {
        // Load the project.json file from the cwd
        let project = Project::new();
        let program = Program::new(project.entry);
        match command {
            "build" => {
                let build_matches = matches.subcommand_matches("build").unwrap();
                let target = build_matches.value_of("target").map(|x| String::from(x));
                program.compile(target)
            }
            "validate" => program.validate(),
            "ast" => {
                for (path, source) in program.files {
                    println!("{}\n{:?}\n", path, source.expressions);
                }
            }
            "fmt" => {
                for (path, source) in program.files {
                    println!("{}\n{:?}", path, source.format());
                }
            }
            _ => unreachable!(),
        }
    }
}
