use clap::{Parser, Subcommand, ValueEnum};
use anyhow::{Result, Context};
use std::io::IsTerminal;
use std::process::Command as SysCommand;

use ruster_env::parser; 
use ruster_env::banner;

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

    /// 🚀 Load variables into the current shell session
    Load {
        /// Path to the .env file
        // 2. Hide automatic default, add it manually to description for compactness
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

    /// 🗑️  Unload variables from the current shell session
    Unload {
        /// Path to the .env file
        // Tight packing here too
        #[arg(default_value = ".env", hide_default_value = true, help = "Path to the .env file [default: .env]")]
        path: String,
        
        /// Print verbose output
        #[arg(short, long)]
        verbose: bool,

        #[arg(long, value_enum, hide = true)]
        shell: Option<ShellType>,
    },
    /// 🔍 List all the env variables in the current session
    ///
    /// If a KEY is provided, it prints only that variable's value.
    /// If no KEY is provided, it lists all variables in a aligned, readable format.
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ShellType {
    Powershell,
    Cmd,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Init { shell } => handle_init(*shell),
        Commands::Load { path, shell, verbose, no_overwrite } => handle_load(path, *shell, *verbose, *no_overwrite)?,
        // Simplified Unload call (no 'force' or 'safe' args)
        Commands::Unload { path, shell, verbose } => handle_unload(path, *shell, *verbose)?,
        Commands::Run { path, command, no_overwrite } => handle_run(path, command, *no_overwrite)?,
        Commands::Show { key } => handle_show( key.clone())?,
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

// --- LOGIC HANDLERS ---

fn handle_run(path: &str, args: &[String], no_overwrite: bool) -> Result<()> {
    let vars = parser::parse_env_file(path)?;

    if args.is_empty() {
        anyhow::bail!("No command provided. Usage: ruster-env run -- <command>");
    }

    let program = &args[0];
    let program_args = &args[1..];

    let mut cmd = SysCommand::new(program);
    cmd.args(program_args);

    for var in vars {
        if no_overwrite && std::env::var(&var.key).is_ok() {
            continue;
        }
        cmd.env(&var.key, &var.value);
    }

    let mut child = cmd.spawn().with_context(|| format!("Failed to spawn command: {}", program))?;
    let status = child.wait()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn handle_init(shell_arg: Option<ShellType>) {
    let shell = shell_arg.unwrap_or_else(detect_shell);
    let exe_path_buf = std::env::current_exe().unwrap_or_default();
    let exe_path = exe_path_buf.display().to_string();

    match shell {
        ShellType::Powershell => {
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
    $eval_commands = @("load", "unload")
    if ($eval_commands -contains $command) {{
        if ($args -contains "--help" -or $args -contains "-h") {{ & $exe $command --help; return }}
        $code = & $exe $command --shell powershell $args[1..$args.Count]
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
IF "%1"=="load" GOTO RunEval
IF "%1"=="unload" GOTO RunEval
GOTO PassThrough
:RunEval
    IF "%2"=="--help" GOTO PassThrough
    IF "%2"=="-h" GOTO PassThrough
    "%EXE%" %1 --shell cmd %2 %3 %4 %5 > "%TEMP%\ruster_tmp.bat"
    CALL "%TEMP%\ruster_tmp.bat"
    DEL "%TEMP%\ruster_tmp.bat"
    EXIT /B 0
:PassThrough
"%EXE%" %*
"#;
             let _ = std::fs::write(&wrapper_path, content);
        }
    }
}

fn handle_load(path: &str, shell_arg: Option<ShellType>, verbose: bool, no_overwrite: bool) -> Result<()> {
    let shell = shell_arg.unwrap_or_else(detect_shell);
    let vars = parser::parse_env_file(path)?;

    match shell {
        ShellType::Powershell => {
            for var in &vars {
                if no_overwrite && std::env::var(&var.key).is_ok() {
                    if verbose { println!("Write-Warning '   [SKIP] {} already exists'; ", var.key); }
                    continue; 
                }
                let safe_val = var.value.replace("'", "''");
                println!("$env:{} = '{}';", var.key, safe_val);
                if verbose { println!("Write-Host '   + {}' -ForegroundColor Gray;", var.key); }
            }
            if !no_overwrite {
                println!("Write-Host '[Ruster] Loaded {} variables' -ForegroundColor Green;", vars.len());
            } else {
                 println!("Write-Host '[Ruster] Loaded variables (Safe Mode)' -ForegroundColor Green;");
            }
        },
        ShellType::Cmd => {
            println!("@echo off");
            for var in &vars {
                if no_overwrite && std::env::var(&var.key).is_ok() {
                    if verbose { println!("ECHO    [SKIP] {} already exists", var.key); }
                    continue;
                }
                println!("SET \"{}={}\"", var.key, var.value);
                if verbose { println!("ECHO    + {}", var.key); }
            }
             if !no_overwrite {
                println!("ECHO [Ruster] Loaded {} variables", vars.len());
            } else {
                println!("ECHO [Ruster] Loaded variables (Safe Mode)");
            }
        }
    }
    Ok(())
}
fn handle_unload(path: &str, shell_arg: Option<ShellType>, verbose: bool) -> Result<()> {
    let shell = shell_arg.unwrap_or_else(detect_shell);
    
    // 1. Parse the file to see what we MIGHT need to unload
    let vars = match parser::parse_env_file(path) {
        Ok(v) => v,
        Err(_) => {
            match shell {
                ShellType::Powershell => println!("Write-Warning 'Could not find {} to unload variables from.'", path),
                ShellType::Cmd => println!("ECHO Could not find {} to unload variables from.", path),
            }
            return Ok(());
        }
    };

    let mut count = 0;

    match shell {
        ShellType::Powershell => {
            for var in &vars {
                // Check if it exists BEFORE counting it
                let exists = std::env::var(&var.key).is_ok();
                
                // We always generate the remove command to be safe (idempotent),
                // but we only count/log it if it was actually there.
                println!("Remove-Item env:\\{} -ErrorAction SilentlyContinue;", var.key);

                if exists {
                    count += 1;
                    if verbose { 
                        println!("Write-Host '   - {}' -ForegroundColor DarkGray;", var.key); 
                    }
                }
            }
            
            if count > 0 {
                println!("Write-Host '[Ruster] Unloaded {} variables' -ForegroundColor Yellow;", count);
            } else {
                println!("Write-Host '[Ruster] No active variables found to unload' -ForegroundColor DarkGray;");
            }
        },
        ShellType::Cmd => {
            println!("@echo off");
            for var in &vars {
                let exists = std::env::var(&var.key).is_ok();

                println!("SET \"{}=\"", var.key);

                if exists {
                    count += 1;
                    if verbose { 
                        println!("ECHO    - {}", var.key); 
                    }
                }
            }

            if count > 0 {
                println!("ECHO [Ruster] Unloaded {} variables", count);
            } else {
                println!("ECHO [Ruster] No active variables found to unload");
            }
        }
    }
    Ok(())
}

fn handle_show(key: Option<String>) -> Result<()> {
    if let Some(target_key) = key {
        // --- Single Variable Mode ---
        // Check the ACTUAL system environment
        match std::env::var(&target_key) {
            Ok(val) => println!("{}", val),
            Err(_) => {
                eprintln!("Error: Variable '{}' is not set in the current environment.", target_key);
                std::process::exit(1);
            }
        }
    } else {
        // --- List Mode (All System Env Vars) ---
        // Collect all system variables
        let vars: Vec<(String, String)> = std::env::vars().filter(|(k, _)| !k.starts_with('=')).collect();

        if vars.is_empty() {
             println!("(No environment variables found)");
             return Ok(());
        }

        // Calculate padding based on the longest key
        let max_key_len = vars.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
        
        println!("\n🖥️  System Environment Variables:");
        println!("{:-<1$}", "", max_key_len + 5); // Separator line

        // Sort them alphabetically for readability
        let mut sorted_vars = vars;
        sorted_vars.sort_by(|a, b| a.0.cmp(&b.0));

        for (k, v) in sorted_vars {
             println!("{:<width$}  {}", k, v, width = max_key_len);
        }
        println!();
    }
    Ok(())
}