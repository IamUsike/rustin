### **Shared config with `Rc`** _(Hard)_

Simulate a configuration object shared between three **services** (structs) using `Rc`.
Each service should read from the same configuration object.
Print the reference count at each step using `Rc::strong_count()`.
Explain in comments why you can't use a regular reference here.
