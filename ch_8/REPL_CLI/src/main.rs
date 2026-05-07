// nothing | ok nvm i'll do it

//To enable repl to read the values of command line arguments we pass to it,
//we’ll need the std::env::args function provided in Rust’s standard library.
//This function returns an iterator of the command line arguments passed.
use std::{
    collections::HashMap,
    io::{self, Write},
};

//just realised, that i dont need args, cos we looping and taking user input

// enum Commands {
//     GET,
//     SET,
//     LIST,
//     DELETE,
//     HISTORY,
// }

//ps : only storing valid commands in history

//each kvp will be stored in a hashmap
fn main() {
    // let args: Vec<String> = env::args().collect();
    // let command = &args[1];

    let mut input = String::new();
    let mut history: Vec<String> = Vec::new();
    let mut db: HashMap<String, String> = HashMap::new();

    loop {
        print!("> ");
        //flush to the stdout immediately | dont wait for newline
        io::stdout().flush().expect("Failed to flush stdout");

        input.clear(); //clear old data to reuse buffer 
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let input: Vec<&str> = input.trim().split_whitespace().collect();
        let command = input[0];
        let input_s = input.join(" ");

        match command {
            "SET" => {
                if input.len() != 3 {
                    println!("invalid args");
                    continue;
                }

                let key = input[1].to_string();
                let value = input[2].to_string();

                db.insert(key, value);
                hist(input_s, &mut history);
            }
            "GET" => {
                if input.len() != 2 {
                    println!("invalid args");
                    continue;
                }

                let key = input[1].to_string();
                if !db.contains_key(&key) {
                    println!("Key doesnt exist");
                    continue;
                }

                let value = db.get(&key).unwrap();
                hist(input_s, &mut history);
                println!("{value}");
            }
            "LIST" => {
                if input.len() != 1 {
                    println!("invalid args");
                    continue;
                }

                hist(input_s, &mut history);

                for (key, value) in &db {
                    println!("{}: {}", key, value);
                }
            }
            "DELETE" => {
                if input.len() != 2 {
                    println!("invalid args");
                    continue;
                }

                let key = input[1];

                match db.remove(key) {
                    None => {
                        println!("Key doesn't exist");
                        continue;
                    }
                    Some(i) => {
                        println!("{i}");
                        hist(input_s, &mut history);
                    }
                }
            }
            "HISTORY" => {
                if input.len() != 1 {
                    println!("invalid args");
                    continue;
                }

                for cmd in &history {
                    println!("{cmd}");
                }
            }
            _ => println!("Invalid Action"),
        };
    }
}

//history is saved in queues. The most recent being stored earlier.
//i feel like storing this in stack could also be done.
//okay so, shift_left if the vector size exceeds 10 and store it.
//Since we only need the last 10 commands
//even though `cmd` is moving here, its not an issue cos it'll again be
//moved to his
fn hist(cmd: String, his: &mut Vec<String>) {
    //o(1)
    if his.len() > 10 {
        his.pop();
    }
    //adding to the beginning of vec takes o(n).
    //If i had to do it in rev, then i'd have to left shift
    //that again takes o(n) guess i have no other choice

    his.insert(0, cmd);
}

//okay fuck it, i thought all of these would need different functions but they are
//simple enough to be written in the main itself
