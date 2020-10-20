pub struct MemoryChunk<T>(Vec<T>);

impl<T> MemoryChunk<T> {
    pub fn new() -> Self { MemoryChunk(Vec::new()) }

    pub fn len(&self) -> usize { self.0.len() }

    pub fn with_capacity(cap : usize) -> Self { MemoryChunk(Vec::with_capacity(cap)) }

    pub fn push(&mut self, x : T) { self.0.push(x) }

    pub fn retain<F : Fn(&T) -> bool>(&mut self, f : F) {
        let mut i = 0;
        while i < self.0.len() {
            if f(&self.0[i]) {
                i += 1;
            } else {
                self.0.swap_remove(i);
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> { self.0.iter() }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> { self.0.iter_mut() }
}
