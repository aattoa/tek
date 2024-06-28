pub trait VecIndex {
    fn get(self) -> usize;
    fn new(index: usize) -> Self;
}

#[derive(Clone, Debug)]
pub struct IndexVec<T, Index: VecIndex> {
    pub underlying: Vec<T>,
    marker: std::marker::PhantomData<Index>,
}

// This impl can not be derived because of the phantom data marker.
impl<T, Index: VecIndex> Default for IndexVec<T, Index> {
    fn default() -> Self {
        Self { underlying: Default::default(), marker: Default::default() }
    }
}

impl<T, Index: VecIndex> std::ops::Index<Index> for IndexVec<T, Index> {
    type Output = T;
    fn index(&self, index: Index) -> &T {
        &self.underlying[index.get()]
    }
}

impl<T, Index: VecIndex> std::ops::IndexMut<Index> for IndexVec<T, Index> {
    fn index_mut(&mut self, index: Index) -> &mut T {
        &mut self.underlying[index.get()]
    }
}

impl<T, Index: VecIndex> IndexVec<T, Index> {
    pub fn push(&mut self, element: T) -> Index {
        self.underlying.push(element);
        Index::new(self.underlying.len() - 1)
    }
}

#[macro_export]
macro_rules! define_index {
    ($visibility:vis $name:ident) => {
        #[derive(Clone, Copy, Default, Debug)]
        $visibility struct $name(usize);
        impl $crate::indexvec::VecIndex for $name {
            fn get(self) -> usize { self.0 }
            fn new(index: usize) -> Self { Self(index) }
        }
    }
}

#[cfg(test)]
mod tests {
    define_index!(MyIndex);

    #[test]
    fn index_vec() {
        let mut vec: super::IndexVec<String, MyIndex> = Default::default();
        let id: MyIndex = vec.push(String::from("hello"));
        assert_eq!(vec[id], String::from("hello"));
    }
}
