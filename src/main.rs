use clap::{CommandFactory, Parser, Subcommand};
use anyhow::Result;
use std::{env, process};

// Import our local modules
use ruster_env::{banner, handlers::{self, ShellType}}; 

#[derive(Parser)]
#[command(name = "ruster-env")]
#[command(bin_name = "ruster-env")]
#[command(version)]
#[command(author)]
#[command(about)]
#[command(before_help = banner::BANNER)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(hide = true)]
    Init {
        #[arg(long)] 
        shell: Option<ShellType>,
    },

    /// 🚀 Load variables from a file into the current shell session
    Load {
        /// Path to the .env file
        #[arg(default_value = ".env", hide_default_value = true, help = "Path to the .env file [default: .env]")]
        path: String,
        
        /// Print verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Do not overwrite variables that are already set in the system
        #[arg(long)]
        no_overwrite: bool,

        #[arg(long, value_enum, hide = true)]
        shell: Option<ShellType>,
    },

    /// ➕ Set a single variable for the current session
    Set {
        /// The Key=Value pair (e.g. DATA=Production)
        // REVERTED: required = true, type is String
        #[arg(required = true, help = "Key=Value pair (e.g. DATA=Production)")]
        pair: String,

        #[arg(short, long)]
        verbose: bool,

        #[arg(long, value_enum, hide = true)]
        shell: Option<ShellType>,
    },

    /// 🗑️  Unload variables defined in a file from the session
    Unload {
        /// Path to the .env file
        #[arg(default_value = ".env", hide_default_value = true, help = "Path to the .env file [default: .env]")]
        path: String,
        
        /// Print verbose output
        #[arg(short, long)]
        verbose: bool,

        #[arg(long, value_enum, hide = true)]
        shell: Option<ShellType>,
    },

    /// ➖ Unset (remove) a single variable from the session
    Unset {
        /// The variable Key to remove
        // REVERTED: required = true, type is String
        #[arg(required = true, help = "The variable name to remove")]
        key: String,

        #[arg(short, long)]
        verbose: bool,

        #[arg(long, value_enum, hide = true)]
        shell: Option<ShellType>,
    },

    /// 🔍 List all the env variables in the current session
    Show {
        /// The specific variable key to show (optional)
        #[arg(required = false)]
        key: Option<String>,
    },

    /// 🏃 Run a command in a clean, isolated environment
    Run {
        /// Path to the .env file
        #[arg(short, long, default_value = ".env", hide_default_value = true, help = "Path to the .env file [default: .env]")]
        path: String,

        /// Do not overwrite variables that are already set in the system
        #[arg(long)]
        no_overwrite: bool,

        /// The command to run
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
        command: Vec<String>,
    },
}

fn main() -> Result<()> {
    // Exit 0 if no args
    if env::args().len() == 1 {
        Cli::command().print_help()?; 
        println!(); 
        process::exit(0); 
    }

    let cli = Cli::parse();
    
    // Detect shell once at startup to avoid repeated checks
    let default_shell = handlers::detect_shell();

    match &cli.command {
        Commands::Init { shell } => 
            handlers::handle_init(*shell),
        
        Commands::Load { path, shell, verbose, no_overwrite } => 
            handlers::handle_load(path, shell.clone().unwrap_or(default_shell), *verbose, *no_overwrite)?,
        
        Commands::Set { pair, verbose, shell } => 
            handlers::handle_set(pair, shell.unwrap_or(default_shell), *verbose)?,

        Commands::Unload { path, shell, verbose } => 
            handlers::handle_unload(path, shell.clone().unwrap_or(default_shell), *verbose)?,
        
        Commands::Unset { key, verbose, shell } => 
            handlers::handle_unset(key, shell.unwrap_or(default_shell), *verbose)?,

        Commands::Run { path, command, no_overwrite } => 
            handlers::handle_run(path, command, *no_overwrite)?,
        
        Commands::Show { key } => 
            handlers::handle_show(key.clone())?,
    }
    Ok(())
}