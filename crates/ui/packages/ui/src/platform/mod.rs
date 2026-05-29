#[cfg(target_os = "ios")]
mod darwin;

#[cfg(target_os = "ios")]
pub fn pick_images() -> Result<(), &'static str> {
    darwin::pick_images()
}

#[cfg(not(target_os = "ios"))]
pub fn pick_images() -> Result<(), &'static str> {
    Err("native image picker is not available on this platform")
}
