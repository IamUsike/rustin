# impl Trait vs dyn Trait — feel the difference

Define a trait Drawable { fn draw(&self) -> String; fn area(&self) -> f64; }. Implement it for Circle, Rectangle, Triangle. Write these two functions: fn render_static(shape: &impl Drawable) -> String and fn render_dynamic(shape: &dyn Drawable) -> String. Now write fn largest_shape(shapes: &[Box<dyn Drawable>]) -> &dyn Drawable that returns the shape with the biggest area. Try writing largest_shape with impl Trait instead — watch it fail. Understand why.

> What this cements: impl Trait = one concrete type, monomorphized, faster. dyn Trait = vtable dispatch, any type, needed for heterogeneous collections. You can't return "one of several possible types" with impl Trait.
