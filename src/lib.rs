extern crate failure;

mod macros;
pub mod troika;
pub mod ftroika;

use std::result;

pub type Result<T> = result::Result<T, failure::Error>;
