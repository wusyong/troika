extern crate failure;
extern crate faster;

mod macros;
pub mod stroika;
pub mod troika;

use std::result;

pub type Result<T> = result::Result<T, failure::Error>;
