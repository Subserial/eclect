use clap::{Arg, ArgGroup, ArgMatches, Command, CommandFactory, Error, FromArgMatches, Parser};
use std::fmt::Debug;
use std::fs;

pub fn exclusive_group(id: &'static str) -> ArgGroup {
    ArgGroup::new(id)
        .multiple(false)
        .conflicts_with(CONFIG_FLAG)
}

#[derive(Debug)]
pub struct Config<T> {
    #[allow(dead_code)]
    pub config_path: Option<String>,
    pub inner: T,
}

pub const CONFIG_FLAG: &'static str = "config_path";
pub const CONFIG_FLAG_SHORT: char = 'c';

impl<T> FromArgMatches for Config<T>
where
    T: clap::Parser + serde::de::DeserializeOwned,
{
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, Error> {
        let Some(path) = matches.get_one::<String>(CONFIG_FLAG) else {
            return T::from_arg_matches(matches).map(|t| Self {
                config_path: None,
                inner: t,
            });
        };
        let raw = fs::read_to_string(path).map_err(|err| {
            Command::new("config").error(clap::error::ErrorKind::Io, format!("io error: {}", err))
        })?;
        let val = toml::from_str::<serde_json::Value>(&raw).map_err(|err| {
            Command::new("config").error(
                clap::error::ErrorKind::Io,
                format!("toml parse error: {}", err),
            )
        })?;
        let serde_json::Value::Object(map) = val else {
            return Err(Command::new("config").error(
                clap::error::ErrorKind::Io,
                String::from("internal parse error: found non-object value"),
            ));
        };
        let flags = value_to_flags(&map).map_err(|err| {
            Command::new("config").error(
                clap::error::ErrorKind::Io,
                format!("toml parse error: {}", err),
            )
        })?;
        Ok(Self {
            config_path: Some(path.clone()),
            inner: T::from_arg_matches(&Self::command().get_matches_from(flags))?,
        })
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), Error> {
        self.inner.update_from_arg_matches(matches)
    }
}

impl<T> CommandFactory for Config<T>
where
    T: Parser,
{
    fn command() -> Command {
        T::command()
            .arg(
                Arg::new(CONFIG_FLAG)
                    .short(CONFIG_FLAG_SHORT)
                    .value_name("FILE")
                    .exclusive(true)
                    .help("Read flags from a TOML file. Exclusive to other arguments."),
            )
    }

    fn command_for_update() -> Command {
        T::command_for_update()
    }
}

fn value_to_flags(map: &serde_json::Map<String, serde_json::Value>) -> Result<Vec<String>, String> {
    [Ok(String::from("--"))]
        .into_iter()
        .chain(map.iter().map(|(key, val)| {
            let key_repr = key.replace('_', "-");
            let val_repr = match val {
                serde_json::Value::Object(_) => {
                    return Err(format!("value of \"{}\" is object", key));
                }
                serde_json::Value::Array(arr) => {
                    if arr.iter().any(|v| v.is_object() || v.is_array()) {
                        return Err(format!("value of \"{}\" is too complex", key));
                    }
                    arr.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                }
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            Ok(format!("--{}={}", key_repr, val_repr))
        }))
        .collect()
}

impl<T> clap::Parser for Config<T> where T: clap::Parser + serde::de::DeserializeOwned {}
