# ruster-env

[![GitHub release](https://img.shields.io/github/v/release/Ashoksk029/ruster-env?color=orange)](https://github.com/Ashoksk029/ruster-env/releases)

> **Bend the environment to your will.**

A blazingly fast, **session-persistent** environment variable manager built ecifically for Windows (PowerShell & CMD).

`ruster-env` injects variables directly into your **current terminal ssion**, making them available for every subsequent command you run—just like ource .env` on Linux.

## Features

* **Session Persistence:** Variables stay loaded until you close the terminal.
* **Windows First:** Native support for **PowerShell** and **Command Prompt**.
* **Safety Rails:** `--no-overwrite` protects your system `PATH` and other itical variables.
* **Smart Interpolation:** Supports variable expansion (e.g., `URL=${HOST}:$ORT}`).
* **Clean Unload:** One command to wipe project variables without restarting ur shell.
* **Zero Dependencies:** Single binary, no Python/Node.js required.

---

## Installation

Currently, `ruster-env` is installed from source.

### 1. Build the Binary
```powershell
cargo build --release
# The binary is now at ./target/release/ruster-env.exe
```

### 2. Shell Setup
`ruster-env` requires a small shell hook to modify your current session.

#### **PowerShell**
1.  Open your profile: `notepad $PROFILE`
2.  Add the following line (replace `PATH_TO_EXE` with the actual path):
    ```powershell
    Invoke-Expression (& "C:\Path\To\ruster-core" init --shell wershell | Out-String)
    ```
3.  Restart PowerShell.

#### **Command Prompt (CMD)**
1.  Run the tool once manually:
    ```cmd
    C:\Path\To\ruster-env init --shell cmd
    ```
2.  This creates a `ruster-env.cmd` wrapper in the same folder.
3.  Add that folder to your system `PATH`.

---

## Usage

### 1. Load Variables
Injects variables from `.env` into your current session.
```powershell
ruster-env load
```
* **Result:** `API_KEY` is now available in your shell.
* **Options:**
    * `--verbose`: See exactly what is being set.
    * `--no-overwrite`: Skips variables that already exist in your system (e., prevents hijacking `USERNAME`).

### 2. Show Variables
Checks what is *actually* live in your system.
```powershell
ruster-env show
```
* **List Mode:** Prints all active system variables (hides internal Windows rs like `=::`).
* **Single Mode:** Prints just the value (perfect for scripts).
    ```powershell
    # Copy DB URL to clipboard
    ruster-env show DB_URL | clip
    ```

### 3. Run (Ephemeral)
Runs a single command with variables loaded, **without** modifying your rrent shell.
```powershell
# Variables exist ONLY for this command
ruster-env run -- npm start
```
* **Note:** Use `--` to separate the tool arguments from your command.

### 4. Unload
Removes variables defined in your `.env` file from the session.
```powershell
ruster-env unload
```

---

## .env Syntax
`ruster-env` supports a robust syntax superset:

```ini
# Comments are supported
PORT=8080
HOST=localhost

# Quotes are stripped automatically
SECRET_KEY="super_secret_value"
SINGLE_QUOTES='works_too'

# Interpolation (References other variables)
# Order matters! Define base vars first.
DATABASE_URL=postgres://${HOST}:${PORT}/mydb
```

## License
MIT