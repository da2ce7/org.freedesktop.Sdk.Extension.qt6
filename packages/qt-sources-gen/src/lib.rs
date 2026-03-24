pub mod client;
pub mod error;
pub mod modules;
pub mod sentinel;
pub mod sources;
pub mod verify;

use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CheckResult {
    /// Sources file matches upstream — nothing to do
    UpToDate,
    /// Sentinel changed — new sources content is provided
    Changed { new_sources: String },
    /// Something went wrong
    Error(String),
}
