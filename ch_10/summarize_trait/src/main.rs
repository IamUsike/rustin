trait Summary {
    fn summarize(&self) -> String;
}

struct Article {
    title: String,
    author: String,
    content: String,
}

struct Tweet {
    username: String,
    tweet: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!(
            "The article is written by: {}, content: {}",
            self.author, self.content
        )
    }
}

impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("The tweet {} is written by: {}", self.username, self.tweet)
    }
}

fn print_summary<T: Summary>(item: &T) {
    println!("{}", item.summarize());
}

fn main() {
    let article = Article {
        title: String::from("tit-le"),
        author: String::from("tity"),
        content: String::from("teeteee"),
    };

    print_summary(&article);

    let tweet = Tweet {
        username: String::from("motle"),
        tweet: String::from("letom?"),
    };

    print_summary(&tweet);
}
