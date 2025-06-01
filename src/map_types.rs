use super::{
    allocator_traits::AllocateExt,
    vec_types::{Vector, FixedVec, Iter, IterMut},
};

use std::cmp::Ordering;

pub struct FixedMap<'a, K, V> {
    vec: FixedVec<'a, (K, V)>,
}

impl<'a, K, V> FixedMap<'a, K, V> {

    pub fn new<A>(
        size: usize,
        allocator: &mut A,
    ) -> Option<Self>
        where
            A: AllocateExt<'a>
    {
        Some(Self {
            vec: FixedVec::new(size, allocator)?
        })
    }

    pub fn size(&self) -> usize {
        self.vec.size()
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn insert(
        &mut self,
        key: K,
        value: V,
    ) -> Option<&mut V> 
        where
            K: PartialOrd 
    {
        if self.vec.len() == 0 {
            return Some(&mut self.vec.insert((key, value), 0)?.1);
        }
        let mut left = 0usize;
        let mut right = self.vec.len();
        while left < right {
            let index = left + (right - left) / 2;
            match self.vec[index].0.partial_cmp(&key)? {
                Ordering::Less => { left = index + 1; continue },
                Ordering::Greater => { right = index; continue },
                Ordering::Equal => return None,
            }
        }
        Some(&mut self.vec.insert((key, value), left)?.1)
    }

    pub fn insert_or_modify<F1, F2>(
        &mut self,
        key: K,
        mut f1: F1,
        mut f2: F2,
    ) -> Option<&mut V> 
        where
            K: PartialOrd,
            F1: FnMut() -> V,
            F2: FnMut(&mut V),
    {
        if self.vec.len() == 0 {
            return Some(&mut self.vec.insert((key, f1()), 0)?.1);
        }
        let mut left = 0usize;
        let mut right = self.vec.len();
        while left < right {
            let index = left + (right - left) / 2;
            match self.vec[index].0.partial_cmp(&key)? {
                Ordering::Less => { left = index + 1; continue },
                Ordering::Greater => { right = index; continue },
                Ordering::Equal => {
                    let v = &mut self.vec[index].1;
                    f2(v);
                    return Some(v)
                }
            }
        }
        Some(&mut self.vec.insert((key, f1()), left)?.1)
    }

    pub fn get(&self, key: &K) -> Option<&V>
        where
            K: PartialOrd 
    {
        let mut left = 0usize;
        let mut right = self.vec.len();
        while left < right {
            let index = left + (right - left) / 2;
            match self.vec[index].0.partial_cmp(&key)? {
                Ordering::Less => { left = index + 1; continue },
                Ordering::Greater => { right = index; continue },
                Ordering::Equal => {
                    let v = &self.vec[index].1;
                    return Some(v)
                }
            }
        }
        None
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
        where
            K: PartialOrd 
    {
        let mut left = 0usize;
        let mut right = self.vec.len();
        while left < right {
            let index = left + (right - left) / 2;
            match self.vec[index].0.partial_cmp(&key)? {
                Ordering::Less => { left = index + 1; continue },
                Ordering::Greater => { right = index; continue },
                Ordering::Equal => {
                    let v = &mut self.vec[index].1;
                    return Some(v)
                }
            }
        }
        None
    }

    pub fn iter(&'a self) -> Iter<'a, (K, V)> {
        self.vec.iter()
    }

    pub fn iter_mut(&'a mut self) -> IterMut<'a, (K, V)> {
        self.vec.iter_mut()
    }
}
