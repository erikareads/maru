use clap::{value_parser, Arg, ArgAction, ArgGroup, Command};
use clap_complete::{generate, Generator, Shell};

use serde_json;
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

enum SerializedFormat {
    YAML,
    JSON,
    TOML,
}

fn parse(path: PathBuf, format: SerializedFormat) -> Result<Command, Box<dyn Error>> {
    let serialized_data = read_to_string(path)?;

    let command: Command = match format {
        SerializedFormat::YAML => {
            serde_yaml::from_str::<clap_serde::CommandWrap>(serialized_data.as_str())
                .expect("parse failed")
                .into()
        }
        SerializedFormat::JSON => {
            serde_json::from_str::<clap_serde::CommandWrap>(serialized_data.as_str())
                .expect("parse failed")
                .into()
        }
        SerializedFormat::TOML => {
            toml::from_str::<clap_serde::CommandWrap>(serialized_data.as_str())
                .expect("parse failed")
                .into()
        }
    };

    Ok(command)
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
                        command = parse(from_toml.clone(), SerializedFormat::TOML).expect("failed");
                    }
                    (_, _, Some(from_yaml), _) => {
                        command = parse(from_yaml.clone(), SerializedFormat::YAML).expect("failed");
                    }
                    (_, _, _, Some(from_json)) => {
                        command = parse(from_json.clone(), SerializedFormat::JSON).expect("failed");
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
