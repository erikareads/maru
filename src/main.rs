use clap::{value_parser, Arg, ArgAction, ArgGroup, Command};
use clap_complete::{generate, Generator, Shell};

use std::error::Error;
use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;

fn build_cli() -> Command {
    Command::new("maru").subcommand(
        Command::new("generate")
            .about("Adds files to myapp")
            .arg(Arg::new("self").long("self").action(ArgAction::SetTrue))
            .arg(
                Arg::new("shell")
                    .long("shell")
                    .action(ArgAction::Set)
                    .value_parser(value_parser!(Shell)),
            )
            .arg(
                Arg::new("from-toml")
                    .long("from-toml")
                    .value_parser(value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("from-yaml")
                    .long("from-yaml")
                    .value_parser(value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("from-json")
                    .long("from-json")
                    .value_parser(value_parser!(PathBuf)),
            )
            .group(ArgGroup::new("generation_type").required(true).args([
                "from-toml",
                "from-yaml",
                "from-json",
                "self",
            ])),
    )
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn parse_toml(path: PathBuf) -> Result<Command, Box<dyn Error>> {
    let my_toml = read_to_string(path)?;

    let app: clap::Command = toml::from_str::<clap_serde::CommandWrap>(my_toml.as_str())
        .expect("parse failed")
        .into();

    Ok(app)
}
fn parse_json(path: PathBuf) -> Result<Command, Box<dyn Error>> {
    let my_json = read_to_string(path)?;

    let app: clap::Command = serde_json::from_str::<clap_serde::CommandWrap>(my_json.as_str())
        .expect("parse failed")
        .into();

    Ok(app)
}
fn parse_yaml(path: PathBuf) -> Result<Command, Box<dyn Error>> {
    let my_yaml = read_to_string(path)?;

    let app: clap::Command = serde_yaml::from_str::<clap_serde::CommandWrap>(my_yaml.as_str())
        .expect("parse failed")
        .into();

    Ok(app)
}

fn main() {
    let matches = build_cli().get_matches();

    match matches.subcommand() {
        Some(("generate", submatches)) => {
            if let Some(generator) = submatches.get_one::<Shell>("shell").copied() {
                let (self_flag, from_toml, from_yaml, from_json) = (
                    submatches.get_flag("self"),
                    submatches.get_one::<PathBuf>("from-toml"),
                    submatches.get_one::<PathBuf>("from-yaml"),
                    submatches.get_one::<PathBuf>("from-json"),
                );
                let mut command: Command;
                match (self_flag, from_toml, from_yaml, from_json) {
                    (true, _, _, _) => {
                        command = build_cli();
                    }
                    (_, Some(from_toml), _, _) => {
                        command = parse_toml(from_toml.clone()).expect("failed");
                    }
                    (_, _, Some(from_yaml), _) => {
                        command = parse_yaml(from_yaml.clone()).expect("failed");
                    }
                    (_, _, _, Some(from_json)) => {
                        command = parse_json(from_json.clone()).expect("failed");
                    }
                    _ => unreachable!(),
                }
                print_completions(generator, &mut command);
            }
        }
        None => {
            _ = build_cli().print_help();
        }
        _ => unreachable!(),
    }
}
