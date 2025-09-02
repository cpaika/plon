use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::ui_dioxus::views::*;
use crate::ui_dioxus::app::Dashboard;

#[derive(Clone, Routable, PartialEq, Debug)]
pub enum Route {
    #[route("/")]
    Home {},
    
    #[route("/goals")]
    Goals {},
    
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