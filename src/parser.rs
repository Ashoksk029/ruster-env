use anyhow::{Result, Context};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

pub fn parse_env_file(path: &str) -> Result<Vec<EnvVar>> {
    let path = Path::new(path);
    let file = File::open(path).with_context(|| format!("Could not open .env file: {:?}", path))?;
    let reader = BufReader::new(file);

    let mut vars = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim();

        // 1. Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // 2. Parse KEY=VALUE
        if let Some((raw_key, raw_value)) = line.split_once('=') {
            let key = raw_key.trim().trim_start_matches("export ").trim();
            let mut value = raw_value.trim();

            // 3. Handle Quotes (Strip " or ' from edges)
            if (value.starts_with('"') && value.ends_with('"')) 
            || (value.starts_with('\'') && value.ends_with('\'')) {
                // Remove first and last char
                value = &value[1..value.len() - 1];
            }

            if !key.is_empty() {
                vars.push(EnvVar {
                    key: key.to_string(),
                    value: value.to_string(),
                });
            }
        } else {
            // Warn about malformed lines if necessary? 
            // For now, we silently ignore lines without '='
            eprintln!("[Ruster] Warning: Line {} skipped (missing '=')", i + 1);
        }
    }

    Ok(vars)
}