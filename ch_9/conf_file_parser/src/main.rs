use std::collections::HashMap;
use std::fs;

enum ConfigError {
    MalformedLine(usize),
    EmptyKey(usize),
    DuplicateKey(String),
}

fn main() {
    let path = "../file.txt"; //depends from where we do `cargo run`
    let op = parse_file(path);

    match op {
        Ok(map) => println!("{:?}", map),
        Err(err) => match err {
            ConfigError::MalformedLine(line) => println!("Malformed line at index {line}"),
            ConfigError::EmptyKey(line) => println!("empty key at index {line}"),
            ConfigError::DuplicateKey(key) => println!("Duplicate Key: {key}"),
        },
    };
}

// 1. return Result<hashmap<String, String>, ConfigError>
//
fn parse_file(path: &str) -> Result<HashMap<String, String>, ConfigError> {
    //cant do &str cos 'lifetimes' | that comes in the next chapter so yea
    let mut kvp: HashMap<String, String> = HashMap::new();

    //reads the whole file contents into a string || can use buffer to read large files
    let contents = fs::read_to_string(path).expect("Unable to read file");
    if contents.trim().len() == 0 {
        panic!("Empty file");
    }

    for (i, val) in contents.lines().enumerate() {
        let line = val.trim();

        //skip comments or empty lines
        if line.starts_with('#') || val.len() == 0 {
            continue;
        } else if line.starts_with('=') {
            return Err(ConfigError::EmptyKey(i));
        }

        if !line.contains('=') {
            return Err(ConfigError::MalformedLine(i));
        }

        let v: Vec<&str> = line.splitn(2, '=').collect();
        //required cos it can be ab=
        if v[1].trim() == "" {
            return Err(ConfigError::MalformedLine(i));
        }

        if kvp.contains_key(v[0]) {
            return Err(ConfigError::DuplicateKey(v[0].to_string()));
        }

        kvp.insert(v[0].to_string(), v[1].to_string());
    }

    Ok(kvp)
}
//username admin
//thisisnotaconfigline
//=
//username
