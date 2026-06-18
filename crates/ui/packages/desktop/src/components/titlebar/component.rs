use dioxus::prelude::*;

#[component]
pub fn WindowTitlebar() -> Element {
    #[cfg(not(feature = "desktop"))]
    {
        return rsx! {};
    }

    #[cfg(all(feature = "desktop", not(any(target_os = "linux", target_os = "windows", target_os = "macos"))))]
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

    #[cfg(all(feature = "desktop", any(target_os = "linux", target_os = "windows")))]
    {
        return rsx! {
            div {
                class: "h-9 shrink-0 border-b border-border/60 bg-card/75 backdrop-blur select-none flex items-center relative",
                onmousedown: move |_| {
                    dioxus::desktop::use_window().drag();
                },
                div {
                    class: "absolute inset-0 pointer-events-none flex items-center justify-center",
                    span { class: "text-[11px] tracking-[0.18em] uppercase text-muted-foreground/80", "Warmmy" }
                }
                div { class: "flex-1" }
                div {
                    class: "flex h-full items-center",
                    onmousedown: move |event| {
                        event.stop_propagation();
                    },
                    button {
                        r#type: "button",
                        class: "w-11 h-full text-muted-foreground hover:text-foreground hover:bg-foreground/5 transition-colors",
                        title: "Minimize",
                        onclick: move |_| dioxus::desktop::use_window().window.set_minimized(true),
                        span { class: "text-sm leading-none", "—" }
                    }
                    button {
                        r#type: "button",
                        class: "w-11 h-full text-muted-foreground hover:text-foreground hover:bg-foreground/5 transition-colors",
                        title: "Maximize",
                        onclick: move |_| dioxus::desktop::use_window().toggle_maximized(),
                        span { class: "text-[11px] leading-none", "□" }
                    }
                    button {
                        r#type: "button",
                        class: "w-11 h-full text-muted-foreground hover:text-white hover:bg-rose-500 transition-colors",
                        title: "Close",
                        onclick: move |_| dioxus::desktop::use_window().close(),
                        span { class: "text-sm leading-none", "✕" }
                    }
                }
            }
        };
    }
}
