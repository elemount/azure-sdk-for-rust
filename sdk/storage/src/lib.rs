#![recursion_limit = "256"]
#![allow(clippy::needless_lifetimes)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate azure_core;

pub use self::core::Error;

#[cfg(feature = "account")]
pub mod account;
#[cfg(feature = "blob")]
pub mod blob;
pub mod core;
#[cfg(feature = "data_lake")]
pub mod data_lake;
#[cfg(feature = "file")]
pub mod file;
#[cfg(feature = "queue")]
pub mod queue;
#[cfg(feature = "table")]
pub mod table;

pub use crate::core::*;
#[cfg(feature = "account")]
pub use account::*;
#[cfg(feature = "blob")]
pub use blob::*;
#[cfg(feature = "data_lake")]
pub use data_lake::*;
#[cfg(feature = "file")]
pub use file::*;
#[cfg(feature = "queue")]
pub use queue::*;
#[cfg(feature = "table")]
pub use table::*;
