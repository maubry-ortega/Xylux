//! # Xylux IDE Main Entry Point
//!
//! Command-line interface and application initialization for Xylux IDE.

use std::path::PathBuf;
use std::process;

use clap::{Arg, ArgMatches, Command};

use tracing::{error, info};

use xylux_ide::{
    BUILD_INFO, Config, ConfigLoader, Result, XyluxError, XyluxIde, features, initialize, shutdown,
};

/// Application name for help text.
const APP_NAME: &str = "xylux-ide";

/// Main entry point.
#[tokio::main]
async fn main() {
    // Initialize core systems
    if let Err(e) = initialize().await {
        eprintln!("Failed to initialize core systems: {}", e);
        process::exit(1);
    }

    // Parse command line arguments
    let matches = create_cli().get_matches();

    // Handle the result
    let result = run_with_matches(matches).await;

    // Shutdown core systems
    if let Err(e) = shutdown().await {
        eprintln!("Failed to shutdown core systems: {}", e);
    }

    // Handle any errors
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Create the CLI application.
fn create_cli() -> Command {
    Command::new(APP_NAME)
        .version(BUILD_INFO.version)
        .about("A comprehensive IDE for Rust development and Alux scripting with Xylux engine integration")
        .long_about(format!(
            "Xylux IDE - A modern IDE for game development\n\n{}\n\nFeatures:\n{}",
            BUILD_INFO,
            create_features_description()
        ))
        .arg(
            Arg::new("file")
                .help("File or directory to open")
                .value_name("PATH")
                .index(1)
        )

        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .help("Path to configuration file")
                .value_name("CONFIG_FILE")
        )
        .arg(
            Arg::new("new")
                .long("new")
                .help("Create a new project")
                .value_name("PROJECT_NAME")
        )
        .arg(
            Arg::new("template")
                .long("template")
                .short('t')
                .help("Project template to use with --new")
                .value_name("TEMPLATE")
                .default_value("rust-basic")
                .requires("new")
        )
        .arg(
            Arg::new("alux-repl")
                .long("alux-repl")
                .help("Start Alux REPL")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("headless")
                .long("headless")
                .help("Run in headless mode (no GUI)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("log-level")
                .long("log-level")
                .help("Set log level")
                .value_name("LEVEL")
                .value_parser(["trace", "debug", "info", "warn", "error"])
                .default_value("info")
        )
        .arg(
            Arg::new("check-deps")
                .long("check-deps")
                .help("Check for required dependencies")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("list-templates")
                .long("list-templates")
                .help("List available project templates")
                .action(clap::ArgAction::SetTrue)
        )
        .subcommand(
            Command::new("build")
                .about("Build operations")
                .arg(
                    Arg::new("target")
                        .help("Build target")
                        .value_name("TARGET")
                        .default_value("debug")
                )
        )
        .subcommand(
            Command::new("run")
                .about("Run project")
                .arg(
                    Arg::new("args")
                        .help("Arguments to pass to the program")
                        .value_name("ARGS")
                        .num_args(0..)
                        .last(true)
                )
        )
        .subcommand(
            Command::new("test")
                .about("Run tests")
                .arg(
                    Arg::new("filter")
                        .help("Test filter")
                        .value_name("FILTER")
                )
        )
        .subcommand(
            Command::new("clean")
                .about("Clean build artifacts")
        )
        .subcommand(
            Command::new("config")
                .about("Configuration management")
                .subcommand(
                    Command::new("show")
                        .about("Show current configuration")
                )
                .subcommand(
                    Command::new("reset")
                        .about("Reset configuration to defaults")
                )
                .subcommand(
                    Command::new("path")
                        .about("Show configuration file paths")
                )
        )
}

/// Run the application with parsed command line arguments.
async fn run_with_matches(matches: ArgMatches) -> Result<()> {
    // Handle dependency check
    if matches.get_flag("check-deps") {
        return check_dependencies().await;
    }

    // Handle template listing
    if matches.get_flag("list-templates") {
        return list_templates().await;
    }

    // Handle subcommands
    if let Some((subcommand, sub_matches)) = matches.subcommand() {
        return handle_subcommand(subcommand, sub_matches).await;
    }

    // Load configuration
    let config = load_configuration(&matches).await?;

    // Handle Alux REPL
    if matches.get_flag("alux-repl") {
        return run_alux_repl().await;
    }

    // Handle new project creation
    if let Some(project_name) = matches.get_one::<String>("new") {
        let template = matches.get_one::<String>("template").unwrap();
        return create_new_project(project_name, template).await;
    }

    // Create IDE instance
    let mut ide = XyluxIde::new(config).await?;

    // Handle file/directory opening
    if let Some(path) = matches.get_one::<String>("file") {
        let path = PathBuf::from(path);
        ide.open(path).await?;
    }

    // Setup signal handling
    setup_signal_handlers(&ide).await?;

    // Run the IDE
    info!("Starting Xylux IDE");
    ide.run().await
}

/// Load configuration from various sources.
async fn load_configuration(matches: &ArgMatches) -> Result<Config> {
    let loader = ConfigLoader::new()?;
    let mut config = loader.load()?;

    // Override with command line arguments
    if let Some(log_level) = matches.get_one::<String>("log-level") {
        config.advanced.log_level = log_level.clone();
    }

    if matches.get_flag("headless") {
        config.ui.show_file_explorer = false;
        config.ui.show_terminal = false;
        config.ui.show_minimap = false;
    }

    // Load custom config file if specified
    if let Some(config_file) = matches.get_one::<String>("config") {
        let config_path = PathBuf::from(config_file);
        if !config_path.exists() {
            return Err(XyluxError::config_error(&config_path, 0, "Configuration file not found"));
        }
        // TODO: Load and merge custom config
    }

    Ok(config)
}

/// Setup signal handlers for graceful shutdown.
async fn setup_signal_handlers(ide: &XyluxIde) -> Result<()> {
    let ide_for_signal = ide.clone();

    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};

            let mut sigterm =
                signal(SignalKind::terminate()).expect("Failed to create SIGTERM handler");
            let mut sigint =
                signal(SignalKind::interrupt()).expect("Failed to create SIGINT handler");

            tokio::select! {
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, shutting down gracefully");
                    if let Err(e) = ide_for_signal.request_shutdown().await {
                        error!("Failed to request shutdown: {}", e);
                    }
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT (Ctrl+C), shutting down gracefully");
                    if let Err(e) = ide_for_signal.request_shutdown().await {
                        error!("Failed to request shutdown: {}", e);
                    }
                }
            }
        }

        #[cfg(windows)]
        {
            use tokio::signal::windows;

            let mut ctrl_c = windows::ctrl_c().expect("Failed to create Ctrl+C handler");
            let mut ctrl_break =
                windows::ctrl_break().expect("Failed to create Ctrl+Break handler");

            tokio::select! {
                _ = ctrl_c.recv() => {
                    info!("Received Ctrl+C, shutting down gracefully");
                    if let Err(e) = ide_for_signal.request_shutdown().await {
                        error!("Failed to request shutdown: {}", e);
                    }
                }
                _ = ctrl_break.recv() => {
                    info!("Received Ctrl+Break, shutting down gracefully");
                    if let Err(e) = ide_for_signal.request_shutdown().await {
                        error!("Failed to request shutdown: {}", e);
                    }
                }
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            // For other platforms, just use Ctrl+C
            let mut ctrl_c = tokio::signal::ctrl_c().expect("Failed to create Ctrl+C handler");

            let _ = ctrl_c.recv().await;
            info!("Received interrupt signal, shutting down gracefully");
            if let Err(e) = ide_for_signal.request_shutdown().await {
                error!("Failed to request shutdown: {}", e);
            }
        }
    });

    Ok(())
}

/// Handle subcommands.
async fn handle_subcommand(subcommand: &str, matches: &ArgMatches) -> Result<()> {
    match subcommand {
        "build" => {
            let target = matches.get_one::<String>("target").unwrap();
            println!("Building target: {}", target);
            // TODO: Implement build logic
            Ok(())
        }
        "run" => {
            println!("Running project");
            // TODO: Implement run logic
            Ok(())
        }
        "test" => {
            if let Some(filter) = matches.get_one::<String>("filter") {
                println!("Running tests with filter: {}", filter);
            } else {
                println!("Running all tests");
            }
            // TODO: Implement test logic
            Ok(())
        }
        "clean" => {
            println!("Cleaning build artifacts");
            // TODO: Implement clean logic
            Ok(())
        }
        "config" => handle_config_subcommand(matches).await,
        _ => Err(XyluxError::Arguments(format!("Unknown subcommand: {}", subcommand))),
    }
}

/// Handle config subcommands.
async fn handle_config_subcommand(matches: &ArgMatches) -> Result<()> {
    if let Some((config_cmd, _)) = matches.subcommand() {
        match config_cmd {
            "show" => {
                let config = ConfigLoader::new()?.load()?;
                let config_toml = toml::to_string_pretty(&config)
                    .map_err(|e| XyluxError::config_error("", 0, e.to_string()))?;
                println!("{}", config_toml);
            }
            "reset" => {
                println!("Resetting configuration to defaults");
                let default_config = Config::default();
                let loader = ConfigLoader::new()?;
                loader.save(&default_config)?;
                println!("Configuration reset successfully");
            }
            "path" => {
                println!("Configuration file locations:");
                if let Some(config_dir) = dirs::config_dir() {
                    println!(
                        "  User config: {}",
                        config_dir.join("xylux-ide").join("config.toml").display()
                    );
                }
                println!("  System config: /etc/xylux-ide/config.toml (Unix)");
                println!("  Project config: ./.xylux-ide/config.toml");
            }
            _ => return Err(XyluxError::Arguments("Unknown config subcommand".to_string())),
        }
    } else {
        return Err(XyluxError::Arguments("No config subcommand specified".to_string()));
    }
    Ok(())
}

/// Check for required dependencies.
async fn check_dependencies() -> Result<()> {
    println!("Checking dependencies...\n");

    let mut all_ok = true;

    // Check language servers
    println!("Language Servers:");
    for server in features::available_language_servers() {
        let status = if which::which(server).is_ok() { "✓" } else { "✗" };
        println!("  {} {}", status, server);
        if status == "✗" {
            all_ok = false;
        }
    }

    // Check build tools
    println!("\nBuild Tools:");
    for tool in features::available_build_tools() {
        let status = if which::which(tool).is_ok() { "✓" } else { "✗" };
        println!("  {} {}", status, tool);
        if status == "✗" && tool != "xylux-cli" {
            all_ok = false;
        }
    }

    // Check optional features
    println!("\nOptional Features:");
    println!("  {} Clipboard support", if features::has_clipboard() { "✓" } else { "✗" });
    println!("  {} Network support", if features::has_network() { "✓" } else { "✗" });
    println!("  {} Debug features", if features::has_debug() { "✓" } else { "✗" });

    if all_ok {
        println!("\n✓ All required dependencies are available");
    } else {
        println!(
            "\n⚠ Some dependencies are missing. The IDE will work with reduced functionality."
        );
    }

    Ok(())
}

/// List available project templates.
async fn list_templates() -> Result<()> {
    println!("Available project templates:\n");

    let templates = vec![
        ("rust-basic", "Basic Rust project with Cargo.toml"),
        ("rust-lib", "Rust library project"),
        ("rust-bin", "Rust binary project"),
        ("xylux-game", "Xylux game project with basic structure"),
        ("xylux-plugin", "Xylux plugin project"),
        ("alux-script", "Standalone Alux script project"),
        ("full-stack", "Full-stack project with Rust backend and frontend"),
    ];

    for (name, description) in templates {
        println!("  {:<15} {}", name, description);
    }

    println!("\nUse --template <name> with --new to create a project from a template.");

    Ok(())
}

/// Create a new project.
async fn create_new_project(project_name: &str, template: &str) -> Result<()> {
    let project_path = PathBuf::from(project_name);

    if project_path.exists() {
        return Err(XyluxError::project_error(format!(
            "Directory '{}' already exists",
            project_name
        )));
    }

    println!("Creating new project '{}' using template '{}'", project_name, template);

    // Create project directory
    std::fs::create_dir_all(&project_path)?;

    match template {
        "rust-basic" => create_rust_basic_project(&project_path, project_name)?,
        "xylux-game" => create_xylux_game_project(&project_path, project_name)?,
        _ => {
            return Err(XyluxError::project_error(format!("Unknown template: {}", template)));
        }
    }

    println!("✓ Project '{}' created successfully", project_name);
    println!("  To open the project, run: xylux-ide {}", project_name);

    Ok(())
}

/// Create a basic Rust project.
fn create_rust_basic_project(path: &PathBuf, name: &str) -> Result<()> {
    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        name
    );
    std::fs::write(path.join("Cargo.toml"), cargo_toml)?;

    // Create src directory and main.rs
    std::fs::create_dir_all(path.join("src"))?;
    std::fs::write(
        path.join("src").join("main.rs"),
        r#"fn main() {
    println!("Hello, world!");
}
"#,
    )?;

    // Create .gitignore
    std::fs::write(
        path.join(".gitignore"),
        r#"/target
Cargo.lock
.env
"#,
    )?;

    Ok(())
}

/// Create a Xylux game project.
fn create_xylux_game_project(path: &PathBuf, name: &str) -> Result<()> {
    // First create basic Rust project
    create_rust_basic_project(path, name)?;

    // Create xylux.toml
    let xylux_toml = format!(
        r#"[project]
name = "{}"
version = "0.1.0"
target = "native"

[alux]
hot_reload = true
optimization_level = 1

[assets]
directory = "assets"

[build]
wasm = false
"#,
        name
    );
    std::fs::write(path.join("xylux.toml"), xylux_toml)?;

    // Create directories
    std::fs::create_dir_all(path.join("scripts"))?;
    std::fs::create_dir_all(path.join("assets"))?;
    std::fs::create_dir_all(path.join("shaders"))?;

    // Create sample Alux script
    std::fs::write(
        path.join("scripts").join("main.aux"),
        r#"// Main Alux script for the game
fn main() {
    print("Hello from Alux!");
}
"#,
    )?;

    Ok(())
}

/// Run Alux REPL.
async fn run_alux_repl() -> Result<()> {
    println!("Starting Alux REPL...");
    println!("Type 'exit' or press Ctrl+D to quit.\n");

    // This is a placeholder implementation
    // In a real implementation, you would integrate with the Alux VM
    loop {
        print!("alux> ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).unwrap() == 0 {
            break; // EOF (Ctrl+D)
        }

        let input = input.trim();
        if input == "exit" || input == "quit" {
            break;
        }

        if input.is_empty() {
            continue;
        }

        // Placeholder evaluation
        println!("=> {}", input);
    }

    println!("Goodbye!");
    Ok(())
}

/// Create features description for help text.
fn create_features_description() -> String {
    let mut features = vec![
        "- Full Rust language support with rust-analyzer",
        "- Native Alux scripting language support",
        "- Xylux engine project management",
        "- Hot-reload capabilities",
        "- WebAssembly compilation",
        "- Integrated terminal and build system",
    ];

    if features::has_clipboard() {
        features.push("- Clipboard integration");
    }

    if features::has_network() {
        features.push("- Network features for extensions");
    }

    features.join("\n")
}
