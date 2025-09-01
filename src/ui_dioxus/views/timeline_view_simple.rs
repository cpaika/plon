use dioxus::prelude::*;

#[component]
pub fn TimelineView() -> Element {
    rsx! {
        div {
            class: "timeline-view",
            h2 { "Timeline View" }
            p { "Timeline will be displayed here" }
        }
    }
}