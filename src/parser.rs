use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct EnvVar {
    pub key: String,
    pub value: String,
}

pub fn parse_env_file(path: &str) -> Result<Vec<EnvVar>> {
    let file_path = Path::new(path);
    
    if !file_path.exists() {
        // Return clear error if file missing
        anyhow::bail!("File not found: {}", path);
    }

    let file = File::open(file_path)
        .with_context(|| format!("Failed to open .env file: {}", path))?;
    
    let reader = BufReader::new(file);
    let mut vars: Vec<EnvVar> = Vec::new();
    let mut var_map: HashMap<String, String> = HashMap::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let mut trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Many .env files use "export VAR=VAL". We must strip "export " 
        // so the key becomes just "VAR".
        if trimmed.starts_with("export ") {
            trimmed = trimmed[7..].trim();
        }

        // Split by first '='
        if let Some((key_part, value_part)) = trimmed.split_once('=') {
            let key = key_part.trim().to_string();
            let raw_value = value_part.trim();

            // 1. Remove quotes if present (e.g., "value" -> value)
            let clean_value = strip_quotes(raw_value);

            // 2. Interpolate (Resolve ${VAR} placeholders)
            // We pass 'var_map' so it can find variables defined in previous lines
            let resolved_value = interpolate(&clean_value, &var_map);

            // 3. Store
            var_map.insert(key.clone(), resolved_value.clone());
            vars.push(EnvVar {
                key,
                value: resolved_value,
            });
        } else {
            eprintln!("Warning: Line {} is malformed (missing '='), skipping.", line_num + 1);
        }
    }

    Ok(vars)
}

/// Removes surrounding "" or '' from a string
fn strip_quotes(s: &str) -> String {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        if s.len() >= 2 {
            return s[1..s.len()-1].to_string();
        }
    }
    s.to_string()
}

/// Replaces ${KEY} with the value from the current map or system env
fn interpolate(value: &str, context: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut chars = value.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(&'{') = chars.peek() {
                // Found "${", verify closing "}"
                chars.next(); // Consume '{'
                
                let mut var_name = String::new();
                let mut closed = false;
                
                // Read until '}'
                while let Some(inner_c) = chars.next() {
                    if inner_c == '}' {
                        closed = true;
                        break;
                    }
                    var_name.push(inner_c);
                }

                if closed {
                    // RESOLUTION LOGIC:
                    // 1. Check variables defined earlier in this file
                    if let Some(local_val) = context.get(&var_name) {
                        result.push_str(local_val);
                    } 
                    // 2. Check System Environment variables (e.g., ${PATH})
                    else if let Ok(sys_val) = std::env::var(&var_name) {
                        result.push_str(&sys_val);
                    } 
                    // 3. Not found? Keep literal "${VAR}" (or leave empty? Standard is usually keep literal or empty)
                    // Let's keep the placeholder to indicate error, or you can use empty string.
                    else {
                        result.push_str(&format!("${{{}}}", var_name));
                    }
                } else {
                    // Malformed (no closing bracket), treat as literal text
                    result.push_str("${");
                    result.push_str(&var_name);
                }
            } else {
                // Just a standalone '$', push it
                result.push('$');
            }
        } else {
            result.push(c);
        }
    }
    result
}