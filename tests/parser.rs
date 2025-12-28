use ruster_env::parser::parse_env_file; // Import from your lib
use std::io::Write;
use tempfile::NamedTempFile;

// Helper to create a temporary .env file
fn create_temp_env(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", content).unwrap();
    file
}

#[test]
fn test_basic_key_value() {
    let file = create_temp_env("KEY=VALUE\nANOTHER=123");
    let vars = parse_env_file(file.path().to_str().unwrap()).unwrap();
    
    assert_eq!(vars.len(), 2);
    assert_eq!(vars[0].key, "KEY");
    assert_eq!(vars[0].value, "VALUE");
}

#[test]
fn test_comments_and_empty_lines() {
    let content = r#"
        # This is a comment
        
        VALID=true
        # Another comment
        SKIP_ME
    "#;
    let file = create_temp_env(content);
    let vars = parse_env_file(file.path().to_str().unwrap()).unwrap();

    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0].key, "VALID");
}

#[test]
fn test_strip_quotes() {
    let content = r#"
        SINGLE='value'
        DOUBLE="value"
    "#;
    let file = create_temp_env(content);
    let vars = parse_env_file(file.path().to_str().unwrap()).unwrap();

    assert_eq!(vars[0].value, "value"); 
    assert_eq!(vars[1].value, "value"); 
}

#[test]
fn test_export_keyword() {
    let file = create_temp_env("export MY_VAR=cool");
    let vars = parse_env_file(file.path().to_str().unwrap()).unwrap();

    assert_eq!(vars[0].key, "MY_VAR");
    assert_eq!(vars[0].value, "cool");
}

#[test]
fn test_interpolation() {
    // Note: We test interpolation via public API now (Integration style)
    let content = r#"
        BASE=http://localhost
        FULL=${BASE}/api
    "#;
    let file = create_temp_env(content);
    let vars = parse_env_file(file.path().to_str().unwrap()).unwrap();

    assert_eq!(vars[1].value, "http://localhost/api");
}