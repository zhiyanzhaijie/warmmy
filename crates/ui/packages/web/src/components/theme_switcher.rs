use dioxus::prelude::*;
use dioxus_icons::lucide::{Moon, Sun};

#[component]
pub fn ThemeSwitcher() -> Element {
    let mut is_dark = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            let eval = document::eval(
                r#"
                const theme = document.documentElement.getAttribute('data-theme') 
                    || (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
                return theme;
                "#,
            );

            if let Ok(val) = eval.await {
                if let Some(theme_str) = val.as_str() {
                    is_dark.set(theme_str == "dark");
                }
            }
        });
    });

    rsx! {
        button {
            class: "inline-flex h-8 w-8 items-center justify-center rounded-full text-muted-foreground transition-colors hover:text-foreground hover:cursor-pointer",
            onclick: move |_| {
                let next_is_dark = !is_dark();
                is_dark.set(next_is_dark);
                let theme_str = if next_is_dark { "dark" } else { "light" };

                let _ = document::eval(&format!(
                    "document.documentElement.setAttribute('data-theme', '{}');",
                    theme_str
                ));
            },
            aria_label: "切换主题",
            if is_dark() {
                Moon {
                    size: 18,
                }
            } else {
                Sun {
                    size: 18,
                }
            }
        }
    }
}
