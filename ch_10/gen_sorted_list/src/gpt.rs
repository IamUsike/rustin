pub struct SortedList<T: Ord> {
    data: Vec<T>,
}

impl<T: Ord> SortedList<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn insert(&mut self, item: T) {
        let pos = self.binary_search_position(&item);
        self.data.insert(pos, item);
    }

    pub fn remove(&mut self, item: &T) -> bool {
        if let Ok(pos) = self.data.binary_search(item) {
            self.data.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn contains(&self, item: &T) -> bool {
        self.data.binary_search(item).is_ok()
    }

    pub fn min(&self) -> Option<&T> {
        self.data.first()
    }

    pub fn max(&self) -> Option<&T> {
        self.data.last()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    fn binary_search_position(&self, item: &T) -> usize {
        match self.data.binary_search(item) {
            Ok(pos) | Err(pos) => pos,
        }
    }
}

fn main() {
    let mut list = SortedList::new();

    list.insert(30);
    list.insert(10);
    list.insert(50);
    list.insert(20);
    list.insert(40);

    println!("List: {:?}", list.as_slice());

    println!("Contains 20: {}", list.contains(&20));
    println!("Contains 25: {}", list.contains(&25));

    println!("Min: {:?}", list.min());
    println!("Max: {:?}", list.max());

    println!("Removed 30: {}", list.remove(&30));
    println!("Removed 100: {}", list.remove(&100));

    println!("List after removal: {:?}", list.as_slice());
}
