use dioxus::prelude::*;
use crate::ui_dioxus::components::ClaudeConfigAdmin;

#[component]
pub fn ClaudeConfigView() -> Element {
    rsx! {
        div { class: "view-container",
            ClaudeConfigAdmin {}
        }
    }
}