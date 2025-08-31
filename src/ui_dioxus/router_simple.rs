use dioxus::prelude::*;
use crate::ui_dioxus::views::*;

#[derive(Clone, Routable, PartialEq, Debug)]
pub enum Route {
    #[route("/")]
    Home {},
    
    #[route("/map")]
    Map {},
    
    #[route("/list")]
    List {},
    
    #[route("/kanban")]
    Kanban {},
    
    #[route("/timeline")]
    Timeline {},
    
    #[route("/gantt")]
    Gantt {},
}

#[component]
fn Home() -> Element {
    rsx! { Dashboard {} }
}

#[component]
fn Map() -> Element {
    rsx! { MapView {} }
}

#[component]
fn List() -> Element {
    rsx! { ListView {} }
}

#[component]
fn Kanban() -> Element {
    rsx! { KanbanView {} }
}

#[component]
fn Timeline() -> Element {
    rsx! { TimelineView {} }
}

#[component]
fn Gantt() -> Element {
    rsx! { GanttView {} }
}