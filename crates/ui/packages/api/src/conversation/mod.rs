#[cfg(feature = "local")]
mod local;
#[cfg(not(feature = "local"))]
mod remote;

#[cfg(feature = "local")]
pub use local::*;
#[cfg(not(feature = "local"))]
pub use remote::*;
