fn main() {
    // Check for command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                println!("Plon - Task Management and Automation System");
                println!();
                println!("Usage: plon [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --help, -h     Show this help message");
                println!("  --version, -v  Show version information");
                println!();
                println!("By default, launches the desktop UI");
            }
            "--version" | "-v" => {
                println!("Plon version {}", env!("CARGO_PKG_VERSION"));
            }
            _ => {
                println!("Unknown option: {}", args[1]);
                println!("Use --help for usage information");
            }
        }
    } else {
        // Launch the desktop UI by default
        use plon::ui_dioxus::App;
        dioxus::launch(App);
    }
}