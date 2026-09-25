//for ease of use input lifetime is 'i and schema lifetime is 's
#[derive(Debug)]
struct StrParser<'i, 's> {
    input: &'i str,
    schema: &'s str,
}

impl<'i, 's> StrParser<'i, 's> {
    fn new(input: &'i str, schema: &'s str) -> Self {
        Self { input, schema }
    }

    fn matches(&self) -> bool {
        self.input.starts_with(self.schema)
    }

    //always assuming that the match exists
    fn remaining(&self) -> &'i str {
        &self.input[self.schema.len()..]
    }
}

fn takeout_schema(a: String) {
    println!("tookout {a}");
}

fn main() {
    let input = "hi my name is chika chika slim shady";
    let schema = "hi".to_string();

    let parser = StrParser::new(input, &schema);

    let matches = parser.matches();
    println!("{matches}");

    let rem = parser.remaining();

    //moved the value of schema
    takeout_schema(schema);

    //remaining is still valid here, cos rthe output lifetime is only
    //tied to the input(field) ka lifetime
    println!("{rem}");

    // println!("{:?}", parser); //cant use parser here cos schema is moved out (as parser contains
    // both input and schema)
}
