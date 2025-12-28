// tests/cli.rs

use assert_cmd::{Command, cargo};
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

// Helper to get our binary command
fn cmd() -> Command {
    let path = cargo::cargo_bin!("ruster-core"); 
    
    // 2. Create the Command from that path
    Command::new(path)
}

// Helper to create a temporary .env file
fn create_temp_env(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", content).unwrap();
    file
}

#[test]
fn test_help_banner() {
    let mut cmd = cmd();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bend the environment to your will")); // Check banner
}

#[test]
fn test_load_output_cmd() {
    let file = create_temp_env("TEST_KEY=123");
    let path = file.path().to_str().unwrap();

    let mut cmd = cmd();
    // We force --shell cmd to check specific output format
    cmd.arg("load")
       .arg(path)
       .arg("--shell").arg("cmd") 
       .assert()
       .success()
       .stdout(predicate::str::contains("SET \"TEST_KEY=123\""));
}

#[test]
fn test_load_output_powershell() {
    let file = create_temp_env("TEST_KEY=abc");
    let path = file.path().to_str().unwrap();

    let mut cmd = cmd();
    // We force --shell powershell
    cmd.arg("load")
       .arg(path)
       .arg("--shell").arg("powershell")
       .assert()
       .success()
       .stdout(predicate::str::contains("$env:TEST_KEY = 'abc';"));
}

#[test]
fn test_unload_cmd() {
    // We need to trick the unloader into thinking the var exists to see output.
    // However, our current 'unload' logic checks std::env::var().
    // Integration tests run in a separate process, so they won't have the var set 
    // unless we set it in the test runner's environment, but assert_cmd spawns a NEW process.
    
    // So, we expect "No active variables found" since the spawned process won't have the var.
    let file = create_temp_env("TO_DELETE=true");
    let path = file.path().to_str().unwrap();

    let mut cmd = cmd();
    cmd.arg("unload")
       .arg(path)
       .arg("--shell").arg("cmd")
       .assert()
       .success()
       .stdout(predicate::str::contains("[Ruster] No active variables found"));
}

#[test]
fn test_run_ephemeral() {
    let file = create_temp_env("RUN_VAR=secret_value");
    let path = file.path().to_str().unwrap();

    // We run a command that prints the env var. 
    // On Windows, `cmd /C echo %VAR%` is the standard way.
    let mut cmd = cmd();
    
    cmd.arg("run")
       .arg("--path").arg(path)
       .arg("cmd") // The command
       .arg("/C")
       .arg("echo %RUN_VAR%")
       .assert()
       .success()
       .stdout(predicate::str::contains("secret_value")); // Assert output contains the value
}

#[test]
fn test_run_no_overwrite() {
    let file = create_temp_env("PATH=NewPath");
    let path = file.path().to_str().unwrap();

    let mut cmd = cmd();
    
    // We try to overwrite PATH. With --no-overwrite, it should keep the System path.
    // The output should NOT be "NewPath".
    cmd.arg("run")
       .arg("--path").arg(path)
       .arg("--no-overwrite")
       .arg("cmd")
       .arg("/C")
       .arg("echo %PATH%")
       .assert()
       .success()
       .stdout(predicate::str::contains("NewPath").not()); // Should NOT contain the override
}

#[test]
fn test_show_list_cmd() {
    let mut cmd = cmd();
    
    // We inject a fake variable into the process environment
    cmd.env("RUSTER_TEST_VAR", "TestValue") 
       .arg("show")
       // No path argument needed anymore!
       .assert()
       .success()
       .stdout(predicate::str::contains("System Environment Variables")) // Check Header
       .stdout(predicate::str::contains("RUSTER_TEST_VAR")) // Check Key
       .stdout(predicate::str::contains("TestValue")); // Check Value
}

#[test]
fn test_show_single_var() {
    let mut cmd = cmd();

    // Test looking up a specific known variable
    cmd.env("SINGLE_LOOKUP_KEY", "SingleValue")
       .arg("show")
       .arg("SINGLE_LOOKUP_KEY") // Positional argument (KEY)
       .assert()
       .success()
       .stdout(predicate::str::contains("SingleValue").and(
           // Ensure it prints ONLY the value (no headers) so it's clean for scripts
           predicate::str::contains("System Environment Variables").not()
       ));
}