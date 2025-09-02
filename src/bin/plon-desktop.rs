fn main() {
    use plon::ui_dioxus::App;
    use dioxus_desktop::{Config, WindowBuilder};
    
    // Configure the window to not always stay on top
    let window = WindowBuilder::new()
        .with_title("Plon - Task Manager")
        .with_resizable(true)
        .with_always_on_top(false)  // Don't stay in foreground
        .with_decorations(true)
        .with_transparent(false);
    
    let config = Config::default()
        .with_window(window);
    
    // Launch the Dioxus desktop app  
    dioxus_desktop::launch::launch(App, vec![], config);
}