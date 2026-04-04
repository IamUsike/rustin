struct Student {
    name: String,
    grades: Vec<f64>,
}

impl Student {
    // cant do
    // for grade in self.grdes
    // cos it moves self.grades and shared refernce or some shit.
    // got to know that .iter only borrows it, leaving the collection untouched

    fn average(&self) -> f64 {
        let mut sum: f64 = 0.0;
        for grade in self.grades.iter() {
            sum += grade;
        }
        let len = self.grades.len() as f64;
        sum / len
    }

    fn max(&self) -> f64 {
        let mut curMax = f64::MIN;
        for grade in self.grades.iter() {
            if *grade > curMax {
                curMax = *grade
            }
        }

        curMax
    }

    fn min(&self) -> f64 {
        let mut curMin = f64::MAX;
        for grade in self.grades.iter() {
            if *grade < curMin {
                curMin = *grade;
            }
        }
        curMin
    }

    fn grade(&self) -> char {
        let avg = self.average();
        let mut grade = ' ';
        if avg <= 3.0 {
            grade = 'D';
        } else if avg <= 6.0 {
            grade = 'C';
        } else if avg <= 9.0 {
            grade = 'B';
        } else {
            grade = 'A';
        }

        grade
    }
}

fn main() {
    let stu1 = Student {
        name: String::from("bruno"),
        grades: vec![8.8, 9.0, 9.1, 9.4, 9.6, 9.9],
    };

    let stu2 = Student {
        name: String::from("Onana"),
        grades: vec![3.1, 4.1, 3.5, 5.5, 7.2, 4.9],
    };

    let stu3 = Student {
        name: String::from("Casemiro"),
        grades: vec![5.5, 6.0, 7.2, 8.3, 8.8, 9.5],
    };

    println!("Name\tAvg\tMax\tMin\tGrade");

    for stu in [&stu1, &stu2, &stu3] {
        println!(
            "{}\t{:.2}\t{:.2}\t{:.2}\t{}",
            stu.name,
            stu.average(),
            stu.max(),
            stu.min(),
            stu.grade()
        );
    }
}
