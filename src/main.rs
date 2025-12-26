use clap::{Parser, Subcommand, ValueEnum};
use anyhow::Result;
use std::io::IsTerminal;

mod parser;

#[derive(Parser)]
#[command(name = "ruster-env")]
#[command(bin_name = "ruster-env")]
#[command(version)]
#[command(author)] 
#[command(about)]  
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ///  Setup command. (Hidden from normal help)
    #[command(hide = true)] 
    Init {
        #[arg(long)] 
        shell: Option<ShellType>,
    },

    /// Loads variables from a .env file.
    Load {
        #[arg(default_value = ".env")]
        path: String,
        
        #[arg(short, long)]
        verbose: bool,

        #[arg(long, value_enum, hide = true)]
        shell: Option<ShellType>,
    },
    /// Handle in future release
    Unload,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ShellType {
    Powershell,
    Cmd,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Init { shell } => handle_init(*shell),
        Commands::Load { path, shell, verbose } => handle_load(path, *shell, *verbose)?,
        Commands::Unload => println!("echo '[Ruster] Unload not implemented yet'"),
    }
    Ok(())
}

fn detect_shell() -> ShellType {
    if std::env::var("PSModulePath").is_ok() {
        ShellType::Powershell
    } else {
        ShellType::Cmd
    }
}

fn handle_init(shell_arg: Option<ShellType>) {
    let shell = shell_arg.unwrap_or_else(detect_shell);
    let exe_path_buf = std::env::current_exe().unwrap_or_default();
    let exe_path = exe_path_buf.display().to_string();

    match shell {
        ShellType::Powershell => {
            // Guard: If run manually, show instructions instead of code
            if std::io::stdout().is_terminal() {
                println!("\n⚠️  Whoops! You are not meant to run this command directly.\n");
                println!("To install ruster-env, add this line to your PowerShell Profile:");
                println!("---------------------------------------------------------------");
                println!("Invoke-Expression (& '{}' init --shell powershell | Out-String)", exe_path);
                println!("---------------------------------------------------------------\n");
                return; 
            }

            print!(r#"
function ruster-env {{
    $exe = "{exe_path}"
    $command = $args[0]
    if ($command -eq "load") {{
        if ($args -contains "--help" -or $args -contains "-h") {{ & $exe load --help; return }}
        $code = & $exe load --shell powershell $args[1..$args.Count]
        Invoke-Expression ($code | Out-String)
    }} else {{
        & $exe $args
    }}
}}
"#, exe_path = exe_path);
        }
        ShellType::Cmd => {
            let mut wrapper_path = exe_path_buf.clone();
            wrapper_path.set_file_name("ruster-env.cmd");

            let content = r#"@echo off
REM ruster-env wrapper
SET "EXE=%~dp0ruster-core.exe"

IF "%1"=="load" (
    IF "%2"=="--help" GOTO PassThrough
    IF "%2"=="-h" GOTO PassThrough
    "%EXE%" load --shell cmd %2 %3 %4 %5 > "%TEMP%\ruster_tmp.bat"
    CALL "%TEMP%\ruster_tmp.bat"
    DEL "%TEMP%\ruster_tmp.bat"
    EXIT /B 0
)
:PassThrough
"%EXE%" %*
"#;
            if let Ok(_) = std::fs::write(&wrapper_path, content) {
                println!("✅ Generated wrapper: {}", wrapper_path.display());
            } else {
                eprintln!("❌ Failed to write wrapper.");
            }
        }
    }
}

fn handle_load(path: &str, shell_arg: Option<ShellType>, verbose: bool) -> Result<()> {
    let shell = shell_arg.unwrap_or_else(detect_shell);
    let vars = parser::parse_env_file(path)?;

    match shell {
        ShellType::Powershell => {
            for var in &vars {
                let safe_val = var.value.replace("'", "''");
                println!("$env:{} = '{}';", var.key, safe_val);
                if verbose {
                    println!("Write-Host '   + {} = {}' -ForegroundColor Gray;", var.key, safe_val);
                }
            }
            println!("Write-Host '[Ruster] Loaded {} variables' -ForegroundColor Green;", vars.len());
        },
        ShellType::Cmd => {
            println!("@echo off");
            for var in &vars {
                println!("SET \"{}={}\"", var.key, var.value);
                if verbose {
                    println!("ECHO    + {}={}", var.key, var.value);
                }
            }
            println!("ECHO [Ruster] Loaded {} variables", vars.len());
        }
    }
    Ok(())
}