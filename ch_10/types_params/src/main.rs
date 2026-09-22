trait Parser<Input, Output> {
    fn parse(&self, input: Input) -> Option<Output>;
}

//we dont actually require the struct to hold any data.
//holds how to parse
struct CsvLineParser;

impl<Input> Parser<Input, Vec<String>> for CsvLineParser
where
    Input: AsRef<str>,
{
    fn parse(&self, i: Input) -> Option<Vec<String>> {
        if i.as_ref().is_empty() {
            return None;
        }

        let res: Vec<String> = i.as_ref().split(",").map(String::from).collect();

        Some(res)
    }
}

fn main() {
    println!("Hello, world!");
}
