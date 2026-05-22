mod chat_view;
mod home_view;
mod me_view;

pub use chat_view::{ChatView, ChatDetailView};
pub use home_view::HomeView;
pub use me_view::MeView;

pub static FIRST_MSG: dioxus::prelude::GlobalSignal<Option<String>> = dioxus::prelude::Signal::global(|| None);
