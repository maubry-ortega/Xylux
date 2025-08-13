//! # Xylux IDE - GUI Main Entry Point
//!
//! Main entry point for the Xylux IDE GUI application

use std::env;

use xylux_ide::core::Config;
use xylux_ide::gui::XyluxIdeApp;

/// Main entry point for Xylux IDE
fn main() -> eframe::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Handle command line arguments
    match args.len() {
        1 => {
            // No arguments - start GUI
            run_gui_app(None)
        }
        2 => {
            match args[1].as_str() {
                "--version" | "-V" => {
                    println!("Xylux IDE {}", env!("CARGO_PKG_VERSION"));
                    std::process::exit(0);
                }
                "--help" | "-h" => {
                    print_help(&args[0]);
                    std::process::exit(0);
                }
                file_path if !file_path.starts_with('-') => {
                    // Single file argument - start GUI with file
                    run_gui_app(Some(file_path.to_string()))
                }
                arg => {
                    eprintln!("Error: Unrecognized option: {}", arg);
                    eprintln!("Use --help for usage information.");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Error: Too many arguments");
            eprintln!("Use --help for usage information.");
            std::process::exit(1);
        }
    }
}

/// Run the GUI application
fn run_gui_app(file_to_open: Option<String>) -> eframe::Result<()> {
    // Initialize logging
    env_logger::init();

    // Configure native options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Xylux IDE")
            .with_icon(load_icon()),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "Xylux IDE",
        options,
        Box::new(move |cc| {
            let config = Config::default();
            let mut app = XyluxIdeApp::new(cc, config);

            // Open file if specified
            if let Some(file_path) = file_to_open {
                let path = std::path::PathBuf::from(file_path);
                if path.exists() {
                    app.open_file(path);
                }
            }

            Ok(Box::new(app))
        }),
    )
}

/// Load application icon
fn load_icon() -> egui::IconData {
    // Create a simple icon (32x32 pixels)
    let icon_width = 32;
    let icon_height = 32;
    let mut icon_pixels = Vec::new();

    // Create a simple gradient icon
    for y in 0..icon_height {
        for x in 0..icon_width {
            let r = (x * 255 / icon_width) as u8;
            let g = (y * 255 / icon_height) as u8;
            let b = 128u8;
            let a = 255u8;

            icon_pixels.extend_from_slice(&[r, g, b, a]);
        }
    }

    egui::IconData { rgba: icon_pixels, width: icon_width, height: icon_height }
}

/// Print help information
fn print_help(program_name: &str) {
    println!("Xylux IDE {}", env!("CARGO_PKG_VERSION"));
    println!("A modern IDE for Rust development and Alux scripting");
    println!();
    println!("USAGE:");
    println!("    {} [OPTIONS] [FILE]", program_name);
    println!();
    println!("ARGS:");
    println!("    <FILE>    File to open");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print help information");
    println!("    -V, --version    Print version information");
    println!();
    println!("GUI CONTROLS:");
    println!("    Ctrl+N           New file");
    println!("    Ctrl+O           Open file");
    println!("    Ctrl+S           Save file");
    println!("    Ctrl+Q           Quit application");
    println!();
    println!("MENU:");
    println!("    File menu provides file operations");
    println!("    Edit menu provides editing operations");
    println!("    View menu controls panel visibility");
    println!("    Help menu shows application information");
}
