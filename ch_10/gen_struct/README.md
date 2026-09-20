# Generic struct with multiple type params

Define a Pair<K, V> struct (like a HashMap entry). Add: fn new(key: K, val: V) -> Self, fn key(&self) -> &K, fn val(&self) -> &V, fn swap(self) -> Pair<V, K>. Then add a method fn display(&self) only when both K and V implement Display — use an impl block with where clause. Test with Pair<String, i32> and Pair<&str, bool>.

> What this cements: Multiple type params, conditional impl blocks (impl<K,V> Pair<K,V> where K: Display, V: Display), where clause syntax vs inline bounds.
