use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

pub mod public;
pub mod response;
pub mod request;
pub mod filter;

#[derive(Clone, Copy, Debug, Serialize, Deserialize,
    Display, EnumString, AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ApiKeyPrefix {
    SkLive,
    PkLive,
}