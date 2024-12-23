mod api;
mod pairs;
mod shim;

use serde_macros;

pub mod error;

pub mod auth {
    include!(concat!(env!("OUT_DIR"), "/lastfm.auth.rs"));
}
pub mod user {
    include!(concat!(env!("OUT_DIR"), "/lastfm.user.rs"));
}
include!(concat!(env!("OUT_DIR"), "/lastfm.rs"));
