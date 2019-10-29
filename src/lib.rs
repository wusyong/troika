extern crate failure;

pub use self::ftroika::*;
pub use self::troika::*;

pub mod ftroika;
mod macros;
pub mod troika;

use core::result;

pub type Result<T> = result::Result<T, failure::Error>;
