## Mini bank account (hard · capstone)

Build an `Account` struct with:

- `owner` (`String`)
- `balance` (`f64`)
- `transaction_history` (`Vec<String>`)

Add methods:

- `deposit(amount)`
- `withdraw(amount) -> Result<(), String>`
- `print_statement()`

Simulate 5 transactions in `main`.

**Hint:**  
`withdraw` returns `Err` if `balance < amount`.  
Log each transaction as a formatted string in the history vector.
