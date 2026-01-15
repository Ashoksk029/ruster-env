# ruster-env

[![GitHub release](https://img.shields.io/github/v/release/Ashoksk029/ruster-env?color=orange)](https://github.com/Ashoksk029/ruster-env/releases)

 > **Bend the environment to your will.**

 A blazingly fast, **session-persistent** environment variable manager built specifically for Windows (PowerShell & CMD).

 `ruster-env` injects variables directly into your **current terminal session**, making them available for every subsequent command you run—just like `source .env` on Linux.

 ## Features

 * **Session Persistence:** Variables stay loaded until you close the terminal.
 * **Windows First:** Native support for **PowerShell** and **Command Prompt**.
 * **Safety Rails:** `--no-overwrite` protects your system `PATH` and other critical variables.
 * **Smart Interpolation:** Supports variable expansion (e.g., `URL=${HOST}:${PORT}`).
 * **Clean Unload:** One command to wipe project variables without restarting your shell.
 * **Zero Dependencies:** Single binary, no Python/Node.js required.

 ## Installation

### Method 1: Winget (Recommended)

The easiest way to install is via the Windows Package Manager.

```powershell
winget install ruster-env
```

### Method 2: Manual (ZIP)

1. Download the latest `ruster-env-windows-x64.zip` from the [Releases](https://github.com/Ashoksk029/ruster-env/releases) page.
2. Extract the files (`ruster-core.exe` and `ruster-env.cmd`) into a folder.
3. Add that folder to your System **PATH**.

## Shell Setup

If you installed via **Winget** or added the **Manual** folder to PATH, CMD users are ready to go. PowerShell users need a one-time profile update.

 #### **PowerShell**

1. Run the init command to get the setup script for your specific installation path:
    ```powershell
    ruster-env init --shell powershell
    ```

2. Copy the output command. It will appear as follows:
    ```powershell
    Invoke-Expression (& "C:\Path\To\ruster-core.exe" init --shell wershell | Out-String)
    ```

3. Open your PowerShell profile:
   ```powershell
   notepad $PROFILE
   ```

4. Paste the copied command into the file and save it.

5. Restart PowerShell.

> [!TIP]   
> *Alternatively, run the copied command in your current powershell session to start using it immediately.*

#### **Command Prompt (CMD)**

No extra setup required if the folder is in your `PATH`. The `ruster-env.cmd` wrapper handles everything automatically.

## Usage

### 1. Load Variables

Injects variables from `.env` into your current session.

 ```powershell
 ruster-env load
 ```

 * **Result:** `API_KEY` is now available in your shell.
 * **Options:**
  * `--verbose`: See exactly what is being set.
  * `--no-overwrite`: Skips variables that already exist in your system (e.g., prevents hijacking `USERNAME`).

 ### 2. Set / Unset Single Variables

 Quickly add or remove a single variable for the current session.

 ```powershell
 # Set a variable
 ruster-env set DATA=Production

 # Remove a variable
 ruster-env unset DATA
 ```

 ### 3. Show Variables

 Checks what is *actually* live in your system.

 ```powershell
 ruster-env show
 ```

 * **List Mode:** Prints all active system variables (hides internal Windows vars like `=::`).
 * **Single Mode:** Prints just the value (perfect for scripts).

   ```powershell
   # Copy DB URL to clipboard
   ruster-env show DB_URL | clip
   ```

 ### 4. Run (Ephemeral)

 Runs a single command with variables loaded, **without** modifying your current shell.

 ```powershell
# Run a command with variables loaded from .env.test file (default=.env) isolated to that process

> ruster-env run --path .env.test -- python -c "import os; print(os.environ['WELCOME_MSG'])"

Hello World!

# Verify the variable did NOT leak into the current session

> ruster-env show WELCOME_MSG

Error: Variable 'WELCOME_MSG' is not set in the current environment.
 ```

 ### 5. Unload

 Removes variables defined in your `.env` file from the session.

 ```powershell
 ruster-env unload
 ```

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