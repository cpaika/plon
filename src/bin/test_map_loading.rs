use dioxus::prelude::*;
use plon::ui_dioxus::views::MapView;

fn main() {
    // Simple app that starts directly with MapView
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    println!("App starting with MapView...");
    rsx! {
        MapView {}
    }
}