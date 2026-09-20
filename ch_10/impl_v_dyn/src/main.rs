/* impl Trait vs dyn Trait
define a trait Drawable {
    fn draw(&self) -> String;
    fn area(&self) -> f64;
}
implement it for circle, triangle and rect.
write these 2 fns:
- fn render_static(shape: impl Drawable) -> String;
- fn render_dynamic(shape: &dyn Drawable) -> String;
after this write:
- fn largest_shape(shapes: &[Box<dyn Drawable>]) -> &dyn Drawable
this fn should return the shape with the biggest area.
-> Try writing largest_shape with impl Trait instead. Why does this fail
*/

use std::fmt::Debug;

trait Drawable {
    fn draw(&self) -> String;
    fn area(&self) -> f64;
}

#[derive(Debug)]
struct Circle {
    radius: f64,
}

#[derive(Debug)]
struct Triangle {
    base: f64,
    height: f64,
}

#[derive(Debug)]
struct Square {
    edge: f64,
}

impl Drawable for Circle {
    fn draw(&self) -> String {
        println!("draw circle");
        format!("Circle of radius {}", self.radius)
    }

    fn area(&self) -> f64 {
        self.radius * 3.14 * self.radius
    }
}

impl Drawable for Square {
    fn draw(&self) -> String {
        println!("draw sq");
        format!("draw a sq with edge {} units", self.edge)
    }

    fn area(&self) -> f64 {
        self.edge * self.edge
    }
}

impl Drawable for Triangle {
    fn draw(&self) -> String {
        println!("not taking dim man");
        String::from("triangle")
    }

    fn area(&self) -> f64 {
        (self.base * self.height) - 1.0 //No mood to impl traits => no div
    }
}

//take any shape that impl's drawable trait
fn render_static(shape: impl Drawable) -> String {
    shape.draw();
    String::from("Shape rendered statically")
}

//dt using static or dynamic rendering would matter here? Cos in
//the simulation, we already know which concrete type (generic) at
//compile time so monomorphization happens.
//dyn is better to use when yk that a type has a particular trait
//but the concrete type isn't known during compile time.
fn render_dynamic(shape: &dyn Drawable) -> String {
    shape.draw();
    String::from("Shape rendered dynamically")
}

fn largest_shape(shapes: &[Box<dyn Drawable>]) -> &dyn Drawable {
    let mut cur_max = 0.0;
    //if not initialized, compile time error: "largest_sh is possibly uninitialized"
    let mut largest_sh: &dyn Drawable = &*shapes[0];

    for shape in shapes {
        if shape.area() > cur_max {
            cur_max = shape.area();
            //each shape is &Box<dyn Drawable> => ** = dyn Drawable
            largest_sh = &**shape;
        }
    }

    largest_sh
}

fn main() {
    let circle = Circle { radius: 4.0 };
    let square = Square { edge: 4.0 };
    let triangle = Triangle {
        base: 4.0,
        height: 4.0,
    };

    //box is required here cos essentially circle, square and triangle are diff types and a
    //slice/vec can only contain same types elements
    let shapes: Vec<Box<dyn Drawable>> =
        vec![Box::new(circle), Box::new(square), Box::new(triangle)];

    let largest = largest_shape(&shapes);
    //smh idk how else to print it
    println!("fin largest: {:?}", largest);
}
