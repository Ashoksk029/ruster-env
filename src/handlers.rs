use anyhow::{Result, Context};
use clap::ValueEnum;
use std::process::{Command as SysCommand};
use std::io::IsTerminal;

use crate::parser;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum ShellType {
    Powershell,
    Cmd,
}

pub fn detect_shell() -> ShellType {
    // Check for PROMPT (Standard CMD variable)
    if std::env::var("PROMPT").is_ok() {
        return ShellType::Cmd;
    }
    // If PSModulePath exists, likely PS context (or just global)
    // But since CMD is covered above, this is safer.
    if std::env::var("PSModulePath").is_ok() {
        return ShellType::Powershell;
    }
    
    // Default to Cmd (Safest lowest common denominator)
    ShellType::Cmd
}

/// Executes a command in a child process with injected variables.
///
/// This is an "Ephemeral" run: variables are only visible to the child process
/// and do not affect the current shell session.
pub fn handle_run(path: &str, args: &[String], no_overwrite: bool) -> Result<()> {
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

/// Generates shell commands to set a single Key=Value pair.
///
/// Handles parsing the pair and outputting the correct syntax for the target shell.
pub fn handle_set(pair: &str, shell: ShellType, verbose: bool) -> Result<()> {
    let parts: Vec<&str> = pair.splitn(2, '=').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid format. Use KEY=VALUE (e.g. DATA=Production)");
    }
    let key = parts[0];
    let value = parts[1];

    match shell {
        ShellType::Powershell => {
            let safe_val = value.replace("'", "''");
            println!("$env:{} = '{}';", key, safe_val);
            if verbose { 
                println!("Write-Host ' + {}' -ForegroundColor Gray;", key);
                println!("Write-Host '[Ruster-Env] Set {}' -ForegroundColor Green;", key);
            }
        },
        ShellType::Cmd => {
            println!("@echo off");
            println!("SET \"{}={}\"", key, value);
            if verbose { 
                println!("ECHO  + {}", key);
                println!("ECHO [Ruster-Env] Set variable {}", key);
            }
        }
    }
    Ok(())
}

/// Generates shell commands to remove a single variable.
///
/// Performs a check to ensure the variable exists before attempting removal.
pub fn handle_unset(key: &str, shell: ShellType, verbose: bool) -> Result<()> {
    // Check if the variable actually exists in the current session
    if std::env::var(key).is_err() {
        match shell {
            ShellType::Powershell => {
                println!("Write-Host '[Ruster-Env] Variable ''{}'' not found in current session' -ForegroundColor Yellow;", key);
            },
            ShellType::Cmd => {
                println!("ECHO [Ruster-Env] Variable '{}' not found in current session", key);
            }
        }
        return Ok(());
    }

    // If it exists, generate the removal code
    match shell {
        ShellType::Powershell => {
            println!("Remove-Item env:\\{} -ErrorAction SilentlyContinue;", key);
            
            if verbose { 
                println!("Write-Host ' - {}' -ForegroundColor DarkGray;", key);
            }
            println!("Write-Host '[Ruster-Env] Unset {}' -ForegroundColor Yellow;", key);
        },
        ShellType::Cmd => {
            println!("@echo off");
            // Setting to empty deletes it in CMD
            println!("SET \"{}=\"", key);
            
            if verbose { 
                println!("ECHO  - {}", key);
            }
            println!("ECHO [Ruster-Env] Unset variable {}", key);
        }
    }
    Ok(())
}

/// Output the shell integration script (Wrapper).
///
/// * PowerShell: Prints a function to `Invoke-Expression`.
/// * CMD: Writes a batch wrapper file next to the binary.
pub fn handle_init(shell_arg: Option<ShellType>) {
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
    $eval_commands = @("load", "unload", "set", "unset")
    if ($eval_commands -contains $command) {{
        if ($args -contains "--help" -or $args -contains "-h") {{ & $exe $command --help; return }}
        $code = & $exe $command --shell powershell $args[1..$args.Count]
        if ($code) {{
            Invoke-Expression ($code | Out-String)
        }}
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
IF "%1"=="set" GOTO RunEval
IF "%1"=="unset" GOTO RunEval
GOTO PassThrough
:RunEval
    IF "%2"=="--help" GOTO PassThrough
    IF "%2"=="-h" GOTO PassThrough
    "%EXE%" %* --shell cmd > "%TEMP%\ruster_tmp.bat"
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

/// Generates shell commands to load variables from a .env file.
///
/// * `no_overwrite`: If true, skips variables that already exist in the system.
/// * `verbose`: Prints details of every variable set.
pub fn handle_load(path: &str, shell: ShellType, verbose: bool, no_overwrite: bool) -> Result<()> {
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
                println!("Write-Host '[Ruster-Env] Loaded {} variables from {}' -ForegroundColor Green;", vars.len(), path);
            } else {
                 println!("Write-Host '[Ruster-Env] Loaded variables from {} (Safe Mode)' -ForegroundColor Green;",path);
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
                println!("ECHO [Ruster-Env] Loaded {} variables from {}", vars.len(), path);
            } else {
                println!("ECHO [Ruster-Env] Loaded variables from {} (Safe Mode)", path);
            }
        }
    }
    Ok(())
}

/// Generates shell commands to unload variables defined in a .env file.
///
/// Filters the list to only generate removal commands for variables that are
/// currently active in the session.
pub fn handle_unload(path: &str, shell: ShellType, verbose: bool) -> Result<()> {
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
                let exists = std::env::var(&var.key).is_ok();
                println!("Remove-Item env:\\{} -ErrorAction SilentlyContinue;", var.key);
                if exists {
                    count += 1;
                    if verbose { println!("Write-Host '   - {}' -ForegroundColor DarkGray;", var.key); }
                }
            }
            if count > 0 {
                println!("Write-Host '[Ruster-Env] Unloaded {} variables' -ForegroundColor Yellow;", count);
            } else {
                println!("Write-Host '[Ruster-Env] No active variables found to unload' -ForegroundColor DarkGray;");
            }
        },
        ShellType::Cmd => {
            println!("@echo off");
            for var in &vars {
                let exists = std::env::var(&var.key).is_ok();
                println!("SET \"{}=\"", var.key);
                if exists {
                    count += 1;
                    if verbose { println!("ECHO    - {}", var.key); }
                }
            }
            if count > 0 {
                println!("ECHO [Ruster-Env] Unloaded {} variables", count);
            } else {
                println!("ECHO [Ruster-Env] No active variables found to unload");
            }
        }
    }
    Ok(())
}

/// Displays environment variables in the console.
///
/// Modes:
/// * Single Key: Prints the raw value (useful for piping).
/// * List All: Prints a formatted table of all system variables (filtering out internal ones).
pub fn handle_show(key: Option<String>) -> Result<()> {
    if let Some(target_key) = key {
        match std::env::var(&target_key) {
            Ok(val) => println!("{}", val),
            Err(_) => {
                eprintln!("Error: Variable '{}' is not set in the current environment.", target_key);
                std::process::exit(1);
            }
        }
    } else {
        let vars: Vec<(String, String)> = std::env::vars().filter(|(k, _)| !k.starts_with('=')).collect();
        if vars.is_empty() {
             println!("(No environment variables found)");
             return Ok(());
        }
        let max_key_len = vars.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
        println!("\n🖥️  System Environment Variables:");
        println!("{:-<1$}", "", max_key_len + 5); 
        let mut sorted_vars = vars;
        sorted_vars.sort_by(|a, b| a.0.cmp(&b.0));
        for (k, v) in sorted_vars {
             println!("{:<width$}  {}", k, v, width = max_key_len);
        }
        println!();
    }
    Ok(())
}