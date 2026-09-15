struct Validated<T>(T);

trait Validate {
    fn validate(&self) -> Result<(), String>;
}

struct Email(String);
struct Port(u16);

impl Validate for Email {
    fn validate(&self) -> Result<(), String> {
        if !self.0.contains('@') || self.0.is_empty() {
            return Err(String::from("Invalid email"));
        }

        Ok(())
    }
}

impl Validate for Port {
    fn validate(&self) -> Result<(), String> {
        //no point taking the upper limit for the port cos that's the upper limit of u16 and it'll
        //overflow ?
        let condition = self.0 < 1;
        if condition {
            return Err(String::from("invalid port"));
        }

        Ok(())
    }
}

//take a generic T that implements the "Validate" trait.
// "a type implements the Validate trait only if there's an impl Validate for T block that provides definitions for all of the trait's required methods"
impl<T: Validate> Validated<T> {
    //returns err if validate fails
    fn new(val: T) -> Result<String, String> {
        match val.validate() {
            Ok(()) => Ok(String::from("all good my ni")),
            Err(err) => Err(err),
        }
    }

    fn inner(&self) -> &T {
        &self.0
    }
}

fn main() {
    let email = Email(String::from("test@test.com"));

    let valid_email = Validated::new(email);

    match valid_email {
        Ok(res) => println!("{res}"),
        Err(err) => println!("{err}"),
    }
}
