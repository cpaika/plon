fn main() {
    use plon::ui_dioxus::App;
    use dioxus_desktop::Config;
    
    // Launch the Dioxus desktop app  
    dioxus_desktop::launch::launch(App, vec![], Config::default());
}