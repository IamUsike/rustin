use std::{
    collections::HashMap,
    io::{self, Write},
};

#[derive(Debug)]
enum Command {
    Set(String, String),
    Get(String),
    Delete(String),
    List,
    History,
    Exit,
    Invalid,
}

fn parse_command(input: &str) -> Command {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();

    if parts.is_empty() {
        return Command::Invalid;
    }

    match parts[0].to_uppercase().as_str() {
        "SET" if parts.len() == 3 => Command::Set(parts[1].to_string(), parts[2].to_string()),
        "GET" if parts.len() == 2 => Command::Get(parts[1].to_string()),
        "DELETE" if parts.len() == 2 => Command::Delete(parts[1].to_string()),
        "LIST" if parts.len() == 1 => Command::List,
        "HISTORY" if parts.len() == 1 => Command::History,
        "EXIT" => Command::Exit,
        _ => Command::Invalid,
    }
}

fn hist(cmd: String, history: &mut Vec<String>) {
    if history.len() >= 10 {
        history.pop(); // remove oldest
    }
    history.insert(0, cmd); // newest at front
}

fn main() {
    let mut db: HashMap<String, String> = HashMap::new();
    let mut history: Vec<String> = Vec::new();
    let mut input = String::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        input.clear();
        io::stdin().read_line(&mut input).unwrap();

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        let command = parse_command(trimmed);

        match command {
            Command::Set(key, value) => {
                db.insert(key, value);
                hist(trimmed.to_string(), &mut history);
                println!("OK");
            }

            Command::Get(key) => match db.get(&key) {
                Some(value) => {
                    println!("{value}");
                    hist(trimmed.to_string(), &mut history);
                }
                None => println!("Key doesn't exist"),
            },

            Command::Delete(key) => match db.remove(&key) {
                Some(value) => {
                    println!("{value}");
                    hist(trimmed.to_string(), &mut history);
                }
                None => println!("Key doesn't exist"),
            },

            Command::List => {
                if db.is_empty() {
                    println!("(empty)");
                } else {
                    for (k, v) in &db {
                        println!("{k}: {v}");
                    }
                }
                hist(trimmed.to_string(), &mut history);
            }

            Command::History => {
                if history.is_empty() {
                    println!("(no history)");
                } else {
                    for cmd in &history {
                        println!("{cmd}");
                    }
                }
            }

            Command::Exit => {
                println!("Bye 👋");
                break;
            }

            Command::Invalid => {
                println!("Invalid command or arguments");
            }
        }
    }
}
