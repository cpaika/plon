use dioxus::prelude::*;

#[component]
pub fn MapView() -> Element {
    rsx! {
        div {
            class: "map-view",
            h2 { "Map View" }
            p { "Interactive task map will be displayed here" }
        }
    }
}

#[component]
pub fn KanbanView() -> Element {
    rsx! {
        div {
            class: "kanban-view",
            h2 { "Kanban Board" }
            p { "Kanban board with actual tasks - see kanban_view_simple.rs for full implementation" }
        }
    }
}

#[component]
pub fn GanttView() -> Element {
    rsx! {
        div {
            class: "gantt-view",
            h2 { "Gantt Chart" }
            p { "Gantt chart with actual tasks - see gantt_view_simple.rs for full implementation" }
        }
    }
}

#[component]
pub fn Dashboard() -> Element {
    rsx! {
        div {
            class: "dashboard",
            h2 { "Dashboard" }
            p { "Dashboard with statistics - see dashboard.rs for full implementation" }
        }
    }
}