struct Student {
    name: String,
    age: u32,
    score: u32,
}

fn main() {}

fn parse(vec: &Vec<&str>) -> Result<Vec<Student>, String> {
    if vec.len() % 3 != 0 || vec.len() == 0 {
        return Err("Invalid Input Provided: Cols Dont match".to_string());
    }

    Err("hello".to_string())

    let v1 = vec.iter().
}

fn build_struct(name: String, age: String, score: String) -> Student {
    let age = age.parse::<u32>().unwrap();
    let score = score.parse::<u32>().unwrap();
    Student { name, age, score }
}
