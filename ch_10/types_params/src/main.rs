// trait Parser<Input, Output> {
//     fn parse(&self, input: Input) -> Option<Output>;
// }

// struct CsvLineParser;

// //output type is pinned, cos the method is constructing the type.
// //input is unpinned generic(inferrerd by the caller type).
// impl<Input: AsRef<str>> Parser<Input, Vec<String>> for CsvLineParser {
//     fn parse(&self, input: Input) -> Option<Vec<String>>
//     where
//         // Input: AsRef<str>, //AsRef is used for cheap ref to ref conversion.
//         //input type needs to be converted to string to parse.
//     {
//         if input.as_ref().is_empty() {
//             return None;
//         }

//         //split returns &str; need to convert this to String
//         let res: Vec<String> = input.as_ref().split(",").map(String::from).collect();

//         Some(res)
//     }
// }

// struct JsonLineParser;

// impl<Input: AsRef<str>> Parser<Input, Vec<(String, String)>> for JsonLineParser {
//     fn parse(&self, input: Input) -> Option<Vec<(String, String)>> {
//         if input.as_ref().trim().is_empty() {
//             return None;
//         }

//         //remove the first and the last curly braces
//         //strip_prefix and suffix are nightly only features
//         let input = input.as_ref().trim().replace("{", "").replace("}", "");

//         // {...} => ... => a:b, c:d => a:b => (a, b)
//         // let res: Vec<(String, String)> = input.as_ref().split(",").split(":").map(String::from).collect();

//         let res: Vec<(String, String)> = input.trim().split(",").filter_map(|kvp| kvp.split_once(":")
//             .map(|(key, value)| (key.to_owned(), value.to_owned()))).collect();

//         Some(res)
//     }
// }

// fn main() {
//     let csv_parse = CsvLineParser;
//     //&str and String impl AsRef<str>
//     let input = "hi, my, name, is, chika, chika, slim, shady";
//     let parsed = run_parser(csv_parse, input);

//     match parsed {
//         Some(v) => println!("{:?}", v),
//         None => println!("plis check the input")
//     }

//     //shadowing
//     let json_parse = JsonLineParser;
//     let input = "{a:b, c:d, e:f}";

//     let parsed = run_parser(json_parse, input);

//     match parsed {
//         Some(v) => println!("{:?}", v),
//         None => println!("Provid a valid json")
//     }
// }

// fn run_parser<Input, Output>(parser: impl Parser<Input, Output>, input: Input) -> Option<Output> {
//     parser.parse(input)
// }

///////////////////////////////////// M2
//go through: https://doc.rust-lang.org/rust-by-example/generics/assoc_items/types.html

trait Parser {
    type Input;
    type Output;

    fn parse(&self, _: Self::Input) -> Option<Self::Output>;
}

struct CsvLineParser;

//unlike generic types, we dont have to express the types
impl Parser for CsvLineParser {
    type Input = String;
    type Output = Vec<String>;

    fn parse(&self, input: Self::Input) -> Option<Self::Output> {
        let res = input.split(",").map(str::to_owned).collect();
        Some(res)
    }
}

struct JsonLineParser;

impl Parser for JsonLineParser {
    type Input = String;
    type Output = Vec<(String, String)>;

    fn parse(&self, input: Self::Input) -> Option<Self::Output> {
        //remove the first and the last curly brace
        let input = input.replace("{", "").replace("}", "");
        //same as prev
        let res: Vec<(String, String)> = input
            .trim()
            .split(",")
            .filter_map(|kvp| {
                kvp.split_once(":")
                    .map(|(key, value)| (key.to_owned(), value.to_owned()))
            })
            .collect();

        Some(res)
    }
}

fn main() {
    let input = "hi, my, name, is, chika, chika, slim, shady";
    let csv_parse = CsvLineParser;

    let parsed = run_parser(csv_parse, input.to_string());
    match parsed {
        None => println!("improper csv input"),
        Some(v) => println!("{:?}", v),
    }

    // shadowing
    let json_parse = JsonLineParser;
    let input = "{a:b, c:d, e:f}";

    let parsed = run_parser(json_parse, input.to_string());

    match parsed {
        Some(v) => println!("{:?}", v),
        None => println!("Provid a valid json"),
    }
}

fn run_parser<Input, Output>(
    parser: impl Parser<Input = Input, Output = Output>,
    input: Input,
) -> Option<Output> {
    parser.parse(input)
}
