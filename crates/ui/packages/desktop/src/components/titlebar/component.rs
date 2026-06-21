use dioxus::prelude::*;

#[component]
pub fn WindowTitlebar() -> Element {
    #[cfg(not(feature = "desktop"))]
    {
        return rsx! {};
    }

    #[cfg(all(
        feature = "desktop",
        not(any(target_os = "linux", target_os = "windows", target_os = "macos"))
    ))]
    {
        return rsx! {};
    }

    #[cfg(all(feature = "desktop", any(target_os = "linux", target_os = "windows")))]
    {
        return rsx! {};
    }

    #[cfg(all(feature = "desktop", target_os = "macos"))]
    {
        return rsx! {
            div {
                class: "fixed top-0 left-1/2 -translate-x-1/2 z-50 w-[180px] h-6 flex items-start justify-center group",
                div {
                    class: "mt-1 h-3 w-24 rounded-b-xl border border-border/60 bg-card/70 backdrop-blur transition-all duration-200 opacity-0 group-hover:opacity-100 group-hover:h-4 group-hover:w-28",
                    button {
                        r#type: "button",
                        class: "w-full h-full cursor-grab active:cursor-grabbing",
                        title: "Drag Window",
                        onmousedown: move |_| {
                            dioxus::desktop::use_window().drag();
                        }
                    }
                }
            }
        };
    }
}
