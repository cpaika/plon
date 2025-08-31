use dioxus::prelude::*;

#[component]
pub fn ListView() -> Element {
    rsx! {
        div {
            class: "list-view",
            h2 { "List View" }
            p { "Task list will be displayed here" }
        }
    }
}