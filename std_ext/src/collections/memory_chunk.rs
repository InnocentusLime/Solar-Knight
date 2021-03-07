use std::slice::{ Iter, IterMut };

pub struct MemoryChunk<T>(Vec<T>);

impl<T> MemoryChunk<T> {
    #[inline]
    pub fn new() -> Self { MemoryChunk(Vec::new()) }

    #[inline]
    pub fn len(&self) -> usize { self.0.len() }

    #[inline]
    pub fn with_capacity(cap : usize) -> Self { MemoryChunk(Vec::with_capacity(cap)) }

    #[inline]
    pub fn push(&mut self, x : T) { self.0.push(x) }
    
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.0.as_mut_slice()
    }

    pub fn retain<F : FnMut(&T) -> bool>(&mut self, mut f : F) {
        let mut i = 0;
        while i < self.0.len() {
            if f(&self.0[i]) {
                i += 1;
            } else {
                self.0.swap_remove(i);
            }
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<T> { self.0.iter() }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T> { self.0.iter_mut() }
}
