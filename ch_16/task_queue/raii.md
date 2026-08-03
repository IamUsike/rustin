## Resource Acquisition is Initialization

- The basic idea of raii is 'when you create an object, allocate memory for that object. And make sure that the memory is destroyed when the object is destroyed'.

- the lifetime of the memory requested is tied to the lifetime of the object using it.
