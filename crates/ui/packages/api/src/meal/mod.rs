#[cfg(feature = "local")]
mod local;
#[cfg(not(feature = "local"))]
mod server;

#[cfg(feature = "local")]
pub use local::*;
#[cfg(not(feature = "local"))]
pub use server::*;
