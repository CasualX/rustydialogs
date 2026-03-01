use super::*;

#[cfg(feature = "appkit")]
mod appkit;
#[cfg(feature = "appkit")]
pub use appkit::*;

#[cfg(not(feature = "appkit"))]
mod osascript;
#[cfg(not(feature = "appkit"))]
pub use osascript::*;
