use std::future::Future;

use dioxus::prelude::*;
#[cfg(feature = "web")]
use serde::de::DeserializeOwned;
#[cfg(feature = "web")]
use serde::Serialize;

#[cfg(feature = "web")]
#[allow(non_snake_case)]
pub fn use_IO<T, F>(future: impl FnMut() -> F + 'static) -> Resource<T>
where
    T: 'static + Serialize + DeserializeOwned,
    F: Future<Output = T> + 'static,
{
    use_server_future(future).unwrap()
}

#[cfg(not(feature = "web"))]
#[allow(non_snake_case)]
pub fn use_IO<T, F>(future: impl FnMut() -> F + 'static) -> Resource<T>
where
    T: 'static,
    F: Future<Output = T> + 'static,
{
    use_resource(future)
}
