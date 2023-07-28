use clap::Arg;
use clap::Command;
use serde::de::DeserializeSeed;
use serde::de::{Error, Visitor};
use serde::Deserialize;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct CommandWrap {
    command: Command,
}

impl From<CommandWrap> for Command {
    fn from(command_wrap: CommandWrap) -> Self {
        command_wrap.command
    }
}

impl From<Command> for CommandWrap {
    fn from(command: Command) -> Self {
        CommandWrap { command }
    }
}

impl<'de> Deserialize<'de> for CommandWrap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(CommandVisitor(Command::new("tmp_name")))
    }
}

struct CommandVisitor(Command);

impl<'de> Visitor<'de> for CommandVisitor {
    type Value = CommandWrap;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Command Map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut command = self.0;
        while let Some(key) = map.next_key::<&str>()? {
            command = match key {
                "after_help" => <Command>::after_help(command, map.next_value::<String>()?),
                "about" => <Command>::about(command, map.next_value::<String>()?),
                "author" => <Command>::author(command, map.next_value::<String>()?),
                "name" => <Command>::name(command, map.next_value::<String>()?),
                "version" => <Command>::version(command, map.next_value::<String>()?),
                "subcommands" => map.next_value_seed(SubCommands(command))?,
                "args" => map.next_value_seed(Args(command))?,
                unknown => return Err(Error::unknown_field(unknown, &["after_help"])),
            };
        }
        Ok(CommandWrap { command })
    }
}

struct SubCommands(Command);
impl<'de> DeserializeSeed<'de> for SubCommands {
    type Value = Command;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de> Visitor<'de> for SubCommands {
    type Value = Command;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Subcommand")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut app = self.0;
        while let Some(name) = map.next_key::<String>()? {
            let sub = map.next_value_seed(NameSeed(name))?;
            app = app.subcommand(sub);
        }
        Ok(app)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut app = self.0;
        while let Some(sub) = seq.next_element_seed(InnerSubCommand)? {
            app = app.subcommand(sub)
        }
        Ok(app)
    }
}

pub struct InnerSubCommand;
impl<'de> Visitor<'de> for InnerSubCommand {
    type Value = Command;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Subcommand Inner")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let k = map
            .next_key()?
            .ok_or_else(|| A::Error::invalid_length(0, &"missing command in subcommand"))?;
        let com = map.next_value_seed(NameSeed(k))?;
        Ok(com.into())
    }
}

impl<'de> DeserializeSeed<'de> for InnerSubCommand {
    type Value = Command;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

struct NameSeed(String);

impl<'de> DeserializeSeed<'de> for NameSeed {
    type Value = CommandWrap;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(CommandVisitor(Command::new(self.0)))
    }
}

impl<'de> DeserializeSeed<'de> for CommandWrap {
    type Value = CommandWrap;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(CommandVisitor(self.command))
    }
}
struct Args(Command);
impl<'de> DeserializeSeed<'de> for Args {
    type Value = Command;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

#[derive(Clone, Copy)]
struct ArgKV(PhantomData<()>);

impl<'de> Visitor<'de> for ArgKV {
    type Value = ArgWrap;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("kv argument")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let name: String = map
            .next_key()?
            .ok_or_else(|| A::Error::missing_field("argument"))?;
        map.next_value_seed(ArgVisitor::new_str(name))
    }
}

impl<'de> DeserializeSeed<'de> for ArgKV {
    type Value = ArgWrap;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for Args {
    type Value = Command;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("args")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let x = ArgKV(PhantomData);
        let mut com = self.0;
        while let Some(a) = seq.next_element_seed(x)? {
            com = com.arg(a);
        }
        Ok(com)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut command = self.0;
        while let Some(name) = map.next_key::<String>()? {
            {
                command = command.arg(map.next_value_seed(ArgVisitor::new_str(name))?);
            }
        }
        Ok(command)
    }
}

impl ArgVisitor {
    fn new_str(v: String) -> Self {
        Self(Arg::new(v))
    }
}

impl<'de> DeserializeSeed<'de> for ArgVisitor {
    type Value = ArgWrap;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

#[derive(Clone, Debug)]
pub struct ArgWrap {
    arg: Arg,
}

impl From<ArgWrap> for Arg {
    fn from(arg_wrap: ArgWrap) -> Self {
        arg_wrap.arg
    }
}

impl From<Arg> for ArgWrap {
    fn from(arg: Arg) -> Self {
        ArgWrap { arg }
    }
}
struct ArgVisitor(Arg);

impl<'de> Visitor<'de> for ArgVisitor {
    type Value = ArgWrap;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Arg Map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut arg: clap::Arg = self.0;
        while let Some(key) = map.next_key::<&str>()? {
            arg = match key {
                // "short" => <Arg>::short(arg, map.next_value::<char>()?),
                "short" => arg.short(map.next_value::<char>()?),
                "long" => <Arg>::long(arg, map.next_value::<String>()?),
                "aliases" => <Arg>::aliases(arg, map.next_value::<Vec<String>>()?),
                unknown => return Err(Error::unknown_field(unknown, &["after_help"])),
            };
        }
        Ok(ArgWrap { arg })
    }
}

#[test]
fn info_toml() {
    const CLAP_TOML: &str = r#"
after_help = "help"
name = "other"
version = "1.0"
author = "erikareads"
about = "test-clap-serde"
[subcommands]
sub1 = { about = "subcommand_1" }
[subcommands.sub2]
about = "subcommand_2"
"#;
    let app: Command = toml::from_str::<CommandWrap>(CLAP_TOML).unwrap().into();
    dbg!(app);
    //assert_eq!(app.get_name(), "other");
}

#[test]
fn info_toml2() {
    const CLAP_TOML: &str = r#"
name = "app_clap_serde"
version = "1.0"
author = "toml_tester"
about = "test-clap-serde"
[subcommands]
sub1 = { about = "subcommand_1" }
[subcommands.sub2]
about = "subcommand_2"
[args]
apple = { short = "a" }
banana = { short = "b", long = "banana", aliases = ["musa_spp"] }
"#;
    let app: Command = toml::from_str::<CommandWrap>(CLAP_TOML).unwrap().into();
    dbg!(app);
    //assert_eq!(app.get_name(), "other");
}
#[test]
fn basic_command() {
    const CLAP_TOML: &str = r#"
name = "app_clap_serde"
"#;
    let app: Command = toml::from_str::<CommandWrap>(CLAP_TOML).unwrap().into();
    let builder = Command::new("app_clap_serde");
    assert_eq!(app.get_name(), builder.get_name());
}
#[test]
fn command_with_args() {
    const CLAP_TOML: &str = r#"
name = "app_clap_serde"
[args]
apple = { short = "a" }
banana = { short = "b", long = "banana", aliases = ["musa_spp"] }
"#;
    let app: Command = toml::from_str::<CommandWrap>(CLAP_TOML).unwrap().into();
    let builder = Command::new("app_clap_serde").args([
        Arg::new("apple").short('a'),
        Arg::new("banana").short('b').long("banana"),
    ]);
    assert_eq!(app.get_name(), builder.get_name());
    let app_args = app.get_arguments();
    let builder_args = builder.get_arguments();
    for (arg1, arg2) in std::iter::zip(app_args, builder_args) {
        assert_eq!(arg1, arg2);
    }
}
