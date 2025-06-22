use crate::vec_types::Iter as VecIter;
use crate::vec_types::IterMut as VecIterMut;

pub struct Iter<'a, Key, Val> {
    key_iter: VecIter<'a, Key>,
    val_iter: VecIter<'a, Val>,
}

impl <'a, Key, Val> Iter<'a, Key, Val> {

    pub unsafe fn new(key_iter: VecIter<'a, Key>, val_iter: VecIter<'a, Val>) -> Self {
        Self {
            key_iter,
            val_iter,
        }
    }
}

impl<'a, Key, Val> Iterator for Iter<'a, Key, Val> {

    type Item = (&'a Key, &'a Val);

    fn next(&mut self) -> Option<Self::Item> {
        Some((
            self.key_iter.next()?, self.val_iter.next()?
        ))
    }
}

impl<'a, Key, Val> DoubleEndedIterator for Iter<'a, Key, Val> {

    fn next_back(&mut self) -> Option<Self::Item> {
        Some((
            self.key_iter.next_back()?, self.val_iter.next_back()?
        ))
    }
}

pub struct IterMut<'a, Key, Val> {
    key_iter: VecIter<'a, Key>,
    val_iter: VecIterMut<'a, Val>,
}

impl <'a, Key, Val> IterMut<'a, Key, Val> {

    pub unsafe fn new(key_iter: VecIter<'a, Key>, val_iter: VecIterMut<'a, Val>) -> Self {
        Self {
            key_iter,
            val_iter,
        }
    }
}

impl<'a, Key, Val> Iterator for IterMut<'a, Key, Val> {

    type Item = (&'a Key, &'a mut Val);

    fn next(&mut self) -> Option<Self::Item> {
        Some((
            self.key_iter.next()?, self.val_iter.next()?
        ))
    }
}

impl<'a, Key, Val> DoubleEndedIterator for IterMut<'a, Key, Val> {

    fn next_back(&mut self) -> Option<Self::Item> {
        Some((
            self.key_iter.next_back()?, self.val_iter.next_back()?
        ))
    }
}
