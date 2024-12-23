use clap::{Arg, ArgMatches, Command, CommandFactory, Error, FromArgMatches, Parser};
use std::fmt::Debug;
use std::fs;

#[derive(Debug)]
pub struct Config<T> {
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
        match matches.get_one::<String>(CONFIG_FLAG) {
            None => T::from_arg_matches(matches).map(|t| Self {
                config_path: None,
                inner: t,
            }),
            Some(path) => {
                let raw = fs::read_to_string(path).map_err(|err| {
                    Command::new("config")
                        .error(clap::error::ErrorKind::Io, format!("io error: {}", err))
                })?;
                let val: T = toml::from_str(&raw).map_err(|err| {
                    Command::new("config").error(
                        clap::error::ErrorKind::Io,
                        format!("toml parse error: {}", err),
                    )
                })?;
                Ok(Self {
                    config_path: Some(path.clone()),
                    inner: val,
                })
            }
        }
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
        T::command().arg(
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

impl<T> clap::Parser for Config<T> where T: clap::Parser + serde::de::DeserializeOwned {}
