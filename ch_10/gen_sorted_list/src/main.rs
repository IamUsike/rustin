//building a generic sorted list

pub trait SortedList<T: Ord> {
    fn insert(&mut self, target: T);
    fn remove(&mut self, target: &T) -> bool;
}

impl<T: Ord> SortedList<T> for Vec<T> {
    fn insert(&mut self, target: T) {
        match self.binary_search(&target) {
            Ok(i) => self.insert(i, target),
            Err(i) => self.insert(i, target),
        };
    }

    fn remove(&mut self, target: &T) -> bool {
        if let Ok(i) = self.binary_search(target) {
            self.remove(i);
            return true;
        } else {
            return false;
        }
    }
}

fn main() {
    let mut arr = vec![10, 20, 30, 40, 50];
    let target = 51;

    //cos theres a defauilt impl for insert on vec and that takes precedence
    //if we do `arr.insert`
    SortedList::insert(&mut arr, target);

    SortedList::remove(&mut arr, &50);

    println!("{:?}", arr)
}

//return index such that index <= target

//gawaar
// fn binary_search(arr: &Vec<i32>, target: i32) -> usize {
//     let mut L = 0;
//     let mut R = arr.len() - 1;
//
//     //fuckin hell i just realized - a bin search will always return the next
//     //largest element if the element isn't found.
//     let mut mid = 0;
//     while L <= R {
//         mid = (L + R) / 2;
//         println!("{}, {}, {}", L, R, mid);
//         if target > arr[mid] {
//             L = mid + 1;
//         } else if target < arr[mid] {
//             R = (mid - 1);
//         } else {
//             break;
//         }
//     }
//
//     mid
// }
