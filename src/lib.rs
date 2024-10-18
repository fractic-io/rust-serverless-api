// This crate is a drop-in replacement for serde_json, but with better error
// messages (specifically, it mentions the name of the field that failed to
// parse).
//
// For this entire library, remap the serde_json crate to use it instead:
extern crate serde_json_path_to_error as serde_json;

pub mod auth;
pub mod errors;
pub mod macros;
pub mod request;
pub mod response;
pub mod routing;
