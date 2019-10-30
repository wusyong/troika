mod constants;
pub mod ftroika;
pub mod troika;

#[cfg(feature = "ftroika")]
pub use ftroika::Ftroika as Troika;

#[cfg(feature = "origin")]
pub use troika::Troika;

pub use sponge_preview::Sponge;

use core::result;
pub type Result<T> = result::Result<T, failure::Error>;
