#[derive(Debug)]

struct Account {
    owner: String,
    balance: f64,
    transaction_history: Vec<String>,
}

impl Account {
    fn deposit(&mut self, amount: f64) {
        self.balance += amount;
        let transction = String::from("+") + &amount.to_string();
        self.transaction_history.push(transction);
    }

    fn withdraw(&mut self, amount: f64) -> Result<(), String> {
        if self.balance - amount < 0.0 {
            return Err(String::from("Insufficient Balance"));
        } else {
            self.balance -= amount;
            let transction = String::from("-") + &amount.to_string();
            self.transaction_history.push(transction);

            Ok(())
        }
    }

    fn print_statement(&self) {
        println!("Statement of {}", self.owner);
        for v in &self.transaction_history {
            println!("{v}");
        }
    }
}

fn main() {
    let mut ac1 = Account {
        owner: String::from("Onana"),
        balance: 1.0,
        transaction_history: vec![],
    };

    ac1.deposit(10.1);
    let with = ac1.withdraw(10.0);
    ac1.print_statement();

    println!("{:?}", ac1);
    println!("{:?}", with);
}
