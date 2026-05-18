use std::collections::HashMap;
use std::fs;

enum ConfigError {
    MalformedLine(usize),
    EmptyKey(usize),
    DuplicateKey(String),
}

fn parse_file(path: &str) -> Result<HashMap<String, String>, ConfigError> {
    let mut kvp: HashMap<String, String> = HashMap::new();

    let contents = fs::read_to_string(path).expect("Unable to read file");

    // ❌ You had a panic for empty file
    // if contents.trim().len() == 0 { panic!("Empty file"); }
    // 👉 Not required by problem, better to just return empty map

    for (i, val) in contents.lines().enumerate() {
        let line = val.trim();

        // ✅ FIX 1: use `line.is_empty()` instead of `val.len() == 0`
        // Your version fails for "   "
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // ❌ You had:
        // else if line.starts_with('=')
        // 👉 This works, BUT is less robust than parsing-first approach

        // ✅ FIX 2: parse FIRST, then validate
        let v: Vec<&str> = line.splitn(2, '=').collect();

        // ❌ You had:
        // if !line.contains('=')
        // 👉 redundant + less clean

        if v.len() != 2 {
            return Err(ConfigError::MalformedLine(i));
        }

        let key = v[0].trim();
        let value = v[1].trim();

        // ✅ FIX 3: EmptyKey should be based on parsed key
        // This covers:
        // "="
        // "   =value"
        // "=   value"
        if key.is_empty() {
            return Err(ConfigError::EmptyKey(i));
        }

        // ❌ You had:
        // if v[1].trim() == "" → MalformedLine
        // 👉 This is WRONG per spec
        // value can be empty: "username=" is valid

        // ✅ FIX 4: duplicate check stays same (your logic was correct)
        if kvp.contains_key(key) {
            return Err(ConfigError::DuplicateKey(key.to_string()));
        }

        kvp.insert(key.to_string(), value.to_string());
    }

    Ok(kvp)
}
