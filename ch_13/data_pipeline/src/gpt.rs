#[derive(Debug)]
struct Student {
    name: String,
    age: u32,
    score: u32,
}

fn parse_row(row: &str) -> Option<Student> {
    let mut parts = row.split(',');

    Some(Student {
        name: parts.next()?.to_string(),
        age: parts.next()?.parse().ok()?,
        score: parts.next()?.parse().ok()?,
    })
}

fn main() {
    let rows = vec![
        "Alice,20,95",
        "Bob,19,67",
        "Charlie,21,88",
        "David,18,72",
        "Eve,22,65",
    ];

    // Parse and filter using iterator chains
    let mut students: Vec<Student> = rows
        .iter()
        .filter_map(|row| parse_row(row))
        .filter(|student| student.score > 70)
        .collect();

    // Sorting requires mutation of the collected Vec
    students.sort_by(|a, b| b.score.cmp(&a.score));

    // Format into a table
    let table = students.iter().fold(
        format!(
            "{:<10} {:<5} {:<5}\n{}",
            "Name",
            "Age",
            "Score",
            "-".repeat(25)
        ),
        |mut acc, student| {
            acc.push_str(&format!(
                "\n{:<10} {:<5} {:<5}",
                student.name, student.age, student.score
            ));
            acc
        },
    );

    println!("{table}");
}
