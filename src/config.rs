use clap::{ArgGroup, ArgMatches, Args, Command, CommandFactory, FromArgMatches};
use std::fmt::{Debug, Display, Formatter};
use std::fs;

pub const CONFIG_FLAG: &'static str = "config-file";
pub const CONFIG_FLAG_SHORT: char = 'c';

#[derive(into_enum::IntoEnum)]
pub enum ConfigError {
    Io(std::io::Error),
    FileParse(toml::de::Error),
    ArgParse(clap::Error),
    Conflict(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(err) => <std::io::Error as Display>::fmt(err, f),
            ConfigError::FileParse(err) => <toml::de::Error as Display>::fmt(err, f),
            ConfigError::ArgParse(err) => <clap::Error as Display>::fmt(err, f),
            ConfigError::Conflict(msg) => write!(f, "{}", msg),
        }
    }
}

pub fn exclusive_group(id: &'static str) -> ArgGroup {
    ArgGroup::new(id).multiple(false).required(true)
}

#[derive(Debug)]
pub struct Config<T> {
    pub config_path: Option<String>,
    pub inner: T,
}

impl<T> Config<T> {
    fn try_get_flags(path: &str) -> Result<Vec<String>, ConfigError> {
        let raw = fs::read_to_string(path)?;
        let val = toml::from_str::<serde_json::Value>(&raw)?;
        let serde_json::Value::Object(map) = val else {
            return Err(Command::new("config").error(
                clap::error::ErrorKind::Io,
                String::from("internal parse error: found non-object value"),
            ).into());
        };
        value_to_flags(&map).map_err(|err| {
            Command::new("config").error(
                clap::error::ErrorKind::Io,
                format!("config error: {}", err),
            ).into()
        })
    }
}

impl<T> FromArgMatches for Config<T>
where
    T: clap::Args + serde::de::DeserializeOwned,
{
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let Some(path) = matches.get_one::<String>(CONFIG_FLAG) else {
            return T::from_arg_matches(matches).map(|t| Self {
                config_path: None,
                inner: t,
            });
        };
        let flags = Self::try_get_flags(path).map_err(|err| {
            let (ty, err) = match err {
                ConfigError::Io(err) => {
                    (clap::error::ErrorKind::Io, format!("io error: {}", err))
                }
                ConfigError::FileParse(err) => {
                    (clap::error::ErrorKind::Io, format!("file error: {}", err))
                }
                ConfigError::ArgParse(err) => {
                    (clap::error::ErrorKind::InvalidValue, format!("argument error: {}", err))
                }
                ConfigError::Conflict(err) => {
                    (clap::error::ErrorKind::InvalidValue, err)
                }
            };
            clap::command!().error(ty, err)
        })?;
        Ok(Self {
            config_path: Some(path.clone()),
            inner: T::from_arg_matches(&Self::command().get_matches_from(flags))?,
        })
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        self.inner.update_from_arg_matches(matches)
    }
}

impl<T> CommandFactory for Config<T>
where
    T: Args,
{
    fn command() -> Command {
        let cmd = T::augment_args(clap::command!());
        cmd.arg(
            clap::Arg::new(CONFIG_FLAG)
                .long(CONFIG_FLAG)
                .short(CONFIG_FLAG_SHORT)
                .exclusive(true)
                .value_name("FILE")
                .help("Read flags from a TOML file. Exclusive to other arguments.")
                .long_help("Read flags from a TOML file. Exclusive to other arguments."),
        )
    }

    fn command_for_update() -> Command {
        T::augment_args_for_update(clap::command!())
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

impl<T> clap::Parser for Config<T> where T: clap::Args + serde::de::DeserializeOwned {}
