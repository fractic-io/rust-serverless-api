// This crate is a drop-in replacement for serde_json, but with better error
// messages (specifically, it mentions the name of the field that failed to
// parse).
//
// For this entire library, remap the serde_json crate to use it instead:
extern crate serde_json_path_to_error as serde_json;

mod auth;
mod constants;
mod crud;
mod errors;
mod macros;
mod request;
mod response;
mod routing;

pub use auth::*;
pub use crud::*;
pub use errors::*;
pub use request::*;
pub use response::*;
pub use routing::*;
