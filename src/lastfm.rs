use base64::Engine;
use prost_lastfm::error::LastFMError;
use prost_lastfm::{AuthService, LastFmService, LastFmServiceAgent, track, user};

pub const PROD_ENDPOINT: &str = "https://ws.audioscrobbler.com/2.0/";
pub const AUTH_ENDPOINT: &str = "https://last.fm/api/auth/";

#[derive(into_enum::IntoEnum)]
pub enum InitError {
    #[into_enum(skip)]
    Internal(String),
    #[into_enum(skip)]
    BadStateFile(String),
    #[into_enum(skip)]
    NeedAuth(String),
    IoError(std::io::Error),
    ReqwestError(reqwest::Error),
    LastFMError(LastFMError),
}

impl From<prost_lastfm::error::Error> for InitError {
    fn from(err: prost_lastfm::error::Error) -> Self {
        match err {
            prost_lastfm::error::Error::LastFM(err) => InitError::LastFMError(err),
            prost_lastfm::error::Error::Reqwest(err) => InitError::ReqwestError(err),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum InitToken {
    Auth(String),
    Session(String),
}

pub fn read_token(token_path: &std::path::Path) -> Result<InitToken, InitError> {
    let raw_token = std::fs::read_to_string(&token_path)?;
    let token_data = base64::engine::general_purpose::STANDARD
        .decode(raw_token)
        .map_err(|err| {
            InitError::BadStateFile(format!(
                "error decoding state file {}: {}",
                token_path.to_string_lossy(),
                err
            ))
        })?;
    let token = serde_json::from_slice::<InitToken>(&token_data).map_err(|err| {
        InitError::BadStateFile(format!(
            "error parsing contents of {}: {}",
            token_path.to_string_lossy(),
            err
        ))
    })?;
    Ok(token)
}

pub fn write_token(token_path: &std::path::Path, init_token: &InitToken) -> Result<(), InitError> {
    let json = serde_json::to_string(init_token)
        .map_err(|err| InitError::Internal(format!("failed to serialize token: {}", err)))?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(json);
    std::fs::write(token_path, encoded).map_err(|err| InitError::from(err))?;
    Ok(())
}

fn generate_auth_request(
    agent: &prost_lastfm::AuthServiceAgent,
    token_path: &std::path::Path,
) -> Result<String, InitError> {
    let request = prost_lastfm::auth::GetTokenRequest {};
    let auth_token = agent.auth_get_token(request)?.token;
    write_token(token_path, &InitToken::Auth(auth_token.clone()))?;
    Ok(auth_token)
}

pub fn build_auth_request(api_key: &str, auth_token: &str) -> url::Url {
    let mut base = url::Url::parse(AUTH_ENDPOINT).unwrap();
    base.query_pairs_mut()
        .append_pair("api_key", api_key)
        .append_pair("token", auth_token);
    base
}

pub fn eprint_auth_request(api_key: &str, auth_token: &str) {
    let request = build_auth_request(api_key, auth_token);
    eprintln!(
        "Navigate to {} to authorize this application, then reopen the program.",
        request.as_str()
    )
}

pub fn activate_session(
    client: reqwest::blocking::Client,
    api_key: &str,
    secret: &str,
    token_path: &std::path::Path,
) -> Result<(LastFmServiceAgent, String), InitError> {
    let auth_agent = prost_lastfm::AuthServiceAgent::new(
        client.clone(),
        api_key.to_string(),
        secret.to_string(),
        AUTH_ENDPOINT.to_string(),
    );
    if !token_path.exists() {
        let auth_token = generate_auth_request(&auth_agent, token_path)?;
        return Err(InitError::NeedAuth(auth_token));
    }
    if token_path.exists() && !token_path.is_file() {
        return Err(InitError::BadStateFile(format!(
            "error accessing {}: not a file",
            token_path.to_string_lossy()
        )));
    }
    let session_token = match read_token(&token_path)? {
        InitToken::Session(session_token) => session_token,
        InitToken::Auth(auth_token) => {
            let request = prost_lastfm::auth::GetSessionRequest {
                token: auth_token.clone(),
            };
            match auth_agent.auth_get_session(request) {
                Err(prost_lastfm::error::Error::Reqwest(err)) => {
                    return Err(InitError::ReqwestError(err));
                }
                Err(prost_lastfm::error::Error::LastFM(err)) => {
                    eprintln!("error returned from requesting session token: {}", err);
                    let auth_token = match err.error {
                        prost_lastfm::error::ErrorCode::AuthenticationFailed => {
                            generate_auth_request(&auth_agent, token_path)?
                        }
                        _ => auth_token,
                    };
                    return Err(InitError::NeedAuth(auth_token));
                }
                Ok(session) => {
                    let Some(session) = session.session else {
                        return Err(InitError::Internal(String::from(
                            "session request error: empty response",
                        )));
                    };
                    write_token(token_path, &InitToken::Session(session.key.clone()))?;
                    session.key
                }
            }
        }
    };
    let lastfm_user_agent = LastFmServiceAgent::new(
        client.clone(),
        api_key.to_string(),
        secret.to_string(),
        PROD_ENDPOINT.to_string(),
    );
    let user = lastfm_user_agent
        .user_get_info(user::GetInfoRequest { user: None }, Some(&session_token))?;
    let Some(user) = user.user else {
        return Err(InitError::Internal(String::from(
            "user request error: empty response",
        )));
    };
    println!("Logged in as {} (url={})", user.name, user.url);
    Ok((lastfm_user_agent, session_token))
}

pub fn now_playing(
    agent: &LastFmServiceAgent,
    session_token: Option<&str>,
) -> Result<Option<prost_lastfm::Track>, prost_lastfm::error::Error> {
    let response = agent.user_get_recent_tracks(
        user::GetRecentTracksRequest {
            limit: Some(1),
            ..Default::default()
        },
        session_token.clone(),
    )?;
    let user::GetRecentTracksResponse {
        recenttracks: Some(prost_lastfm::RecentTracks { track: tracks, .. }),
    } = response
    else {
        return Ok(None);
    };
    let track = tracks.first().and_then(|track| match &track.attr {
        Some(track::Attributes {
            nowplaying: Some(true),
            ..
        }) => Some(track.clone()),
        _ => None,
    });
    Ok(track)
}
