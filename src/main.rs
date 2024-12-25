mod config;
mod discord;
mod lastfm;

use clap::{CommandFactory, Parser};
use config::Config;
use discord_rich_presence::DiscordIpc;
use http::header::USER_AGENT;
use std::time::Duration;

#[derive(clap::Args, serde::Deserialize, Debug)]
#[clap(author, version, about, long_about = None)]
struct ArgumentConfig {
    /// Persistent storage location (Last.fm session token)
    #[clap(short, long, default_value_t = workdir_default())]
    workdir: String,
    /// Seconds between Last.fm queries for now playing.
    #[clap(short, long, default_value_t = 15)]
    query_interval: u64,
    /// The Discord app ID to use.
    /// Required unless --discord-app-id-file is specified.
    #[clap(long)]
    discord_app_id: Option<String>,
    /// A file containing the Discord app ID to use.
    /// Required unless --discord-app-id is specified.
    #[clap(long)]
    discord_app_id_file: Option<String>,
    /// The Last.fm API key to use.
    /// Required unless --lastfm--api-key-file is specified.
    #[clap(long)]
    lastfm_api_key: Option<String>,
    /// A file containing the Last.fm API key to use.
    /// Required unless --lastfm--api-key is specified.
    #[clap(long)]
    lastfm_api_key_file: Option<String>,
    /// The Last.fm API secret to use.
    /// Required unless --lastfm-secret-file is specified.
    #[clap(long)]
    lastfm_secret: Option<String>,
    /// A file containing the Last.fm API secret to use.
    /// Required unless --lastfm-secret is specified.
    #[clap(long)]
    lastfm_secret_file: Option<String>,
}

fn workdir_default() -> String {
    directories::ProjectDirs::from("com", "Panoptic", "Eclect")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or(std::path::Path::new("~/.local/share/eclect").to_path_buf())
        .to_string_lossy()
        .into_owned()
}

fn file_or_string(path: Option<String>, path_name: &str, arg: Option<String>, arg_name: &str) -> Result<String, config::ConfigError> {
    match (path, arg) {
        (Some(path), _) => {
            let path = resolve_path::PathResolveExt::try_resolve(&path)?;
            Ok(std::fs::read_to_string(path)?)
        }
        (_, Some(arg)) => Ok(arg),
        _ => Err(config::ConfigError::Conflict(format!(r#"must specify either {} or {}"#, path_name, arg_name))),
    }
}

struct ProgramConfig {
    workdir: String,
    query_interval: u64,
    discord_app_id: String,
    lastfm_api_key: String,
    lastfm_secret: String,
}

impl ArgumentConfig {
    fn resolve(self) -> Result<ProgramConfig, config::ConfigError> {
        Ok(ProgramConfig {
            workdir: self.workdir,
            query_interval: self.query_interval,
            discord_app_id: file_or_string(self.discord_app_id_file, "discord-app-id-file", self.discord_app_id, "discord-app-id")?,
            lastfm_api_key: file_or_string(self.lastfm_api_key_file, "lastfm-api-key-file", self.lastfm_api_key, "lastfm-api-key")?,
            lastfm_secret: file_or_string(self.lastfm_secret_file, "lastfm-secret-file", self.lastfm_secret, "lastfm-secret")?,
        })
    }
}

fn main() -> Result<(), String> {
    let base_config: Config<ArgumentConfig> = Config::parse();
    let ProgramConfig {
        workdir,
        query_interval,
        discord_app_id,
        lastfm_api_key,
        lastfm_secret,
    } = base_config
        .inner
        .resolve()
        .map_err(|err| {
            Config::<ArgumentConfig>::command().error(
                clap::error::ErrorKind::InvalidValue,
                format!("configuration error: {}", err),
            ).exit()
        }).unwrap();
    let work_path = std::path::Path::new(&workdir);
    if work_path.exists() && !work_path.is_dir() {
        return Err(format!(
            "error accessing data: workdir {} is not a directory",
            &workdir
        ));
    }
    if !work_path.exists() {
        if let Err(err) = std::fs::create_dir_all(work_path) {
            return Err(format!("error creating workdir: {}", err));
        }
    }

    let mut discord_client =
        discord::activate(&discord_app_id).map_err(|err| format!("discord ipc error: {}", err))?;

    let token_path = work_path.join("token.bin");
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .unwrap();
    let (agent, session_key) =
        lastfm::activate_session(client.clone(), &lastfm_api_key, &lastfm_secret, &token_path)
            .map_err(|err| match err {
                lastfm::InitError::Internal(err) => format!("internal error: {}", err),
                lastfm::InitError::BadStateFile(err) => format!("error with state file: {}", err,),
                lastfm::InitError::NeedAuth(auth_token) => {
                    lastfm::eprint_auth_request(&lastfm_api_key, &auth_token);
                    String::from("unauthorized")
                }
                lastfm::InitError::IoError(err) => format!("io error: {}", err),
                lastfm::InitError::ReqwestError(err) => format!("request error: {}", err),
                lastfm::InitError::LastFMError(err) => {
                    format!("error response from server: {}", err)
                }
            })?;

    loop {
        let track = lastfm::now_playing(&agent, Some(&session_key));
        match track {
            Err(err) => println!("Error querying now playing: {}", err),
            Ok(track) => match discord::set_track(&mut discord_client, track) {
                Ok(None) => println!("No track playing"),
                Ok(Some(desc)) => println!("Now playing: {}", desc),
                Err(err) => {
                    println!("Error setting activity: {}", err);
                    discord_client
                        .reconnect()
                        .map_err(|err| format!("discord error: {}", err))?
                }
            },
        }
        std::thread::sleep(Duration::from_secs(query_interval));
    }
}
