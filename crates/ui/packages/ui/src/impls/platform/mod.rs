mod desktop;
mod mobile;
mod web;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformFlavor {
    Web,
    Desktop,
    Mobile,
}

impl PlatformFlavor {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Desktop => "desktop",
            Self::Mobile => "mobile",
        }
    }
}

#[cfg(target_arch = "wasm32")]
use web as active_platform;

#[cfg(all(not(target_arch = "wasm32"), any(target_os = "android", target_os = "ios")))]
use mobile as active_platform;

#[cfg(all(
    not(target_arch = "wasm32"),
    not(any(target_os = "android", target_os = "ios"))
))]
use desktop as active_platform;

pub fn current_platform() -> PlatformFlavor {
    active_platform::current_platform()
}
