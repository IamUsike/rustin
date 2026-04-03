struct Rectangle {
    width: u64,
    height: u64,
}

impl Rectangle {
    fn area(&self) -> u64 {
        self.width * self.height
    }

    fn perimeter(&self) -> u64 {
        2 * (self.width + self.height)
    }

    fn is_square(&self) -> bool {
        self.width == self.height
    }

    fn can_hold(&self, other: &Rectangle) -> bool {
        (self.width >= other.width) && (self.height >= other.height)
    }
}

fn main() {
    let rect = Rectangle {
        width: 10,
        height: 10,
    };

    let rect1 = Rectangle {
        width: 20,
        height: 10,
    };

    println!("Area: {}", rect.area());
    println!("Perimeter: {}", rect.perimeter());
    println!("is_square: {}", rect.is_square());
    println!("can hold: {}", rect.can_hold(&rect1));
}
