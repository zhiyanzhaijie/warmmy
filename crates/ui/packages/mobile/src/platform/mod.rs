#[cfg(target_os = "android")]
mod android;

pub fn init() {
    #[cfg(target_os = "android")]
    android::init();
}
