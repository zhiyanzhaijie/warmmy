use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct MotionWrapperProps {
    #[props(default)]
    pub class: Option<String>,
    pub children: Element,
}

#[component]
pub fn MotionDiv(props: MotionWrapperProps) -> Element {
    rsx! {
        div {
            class: props.class.unwrap_or_default(),
            {props.children}
        }
    }
}

#[component]
pub fn MotionP(props: MotionWrapperProps) -> Element {
    rsx! {
        div {
            class: props.class.unwrap_or_default(),
            {props.children}
        }
    }
}

#[component]
pub fn MotionH2(props: MotionWrapperProps) -> Element {
    rsx! {
        div {
            class: props.class.unwrap_or_default(),
            {props.children}
        }
    }
}

#[component]
pub fn MotionSection(props: MotionWrapperProps) -> Element {
    rsx! {
        div {
            class: props.class.unwrap_or_default(),
            {props.children}
        }
    }
}
