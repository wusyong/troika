extern crate failure;

pub use self::ftroika::*;
pub use self::troika::*;

mod constants;
pub mod ftroika;
pub mod troika;

#[cfg(feature = "ftroika")]
pub use ftroika::Ftroika as Troika;

#[cfg(feature = "origin")]
pub use troika::Troika;

use core::result;
pub type Result<T> = result::Result<T, failure::Error>;
