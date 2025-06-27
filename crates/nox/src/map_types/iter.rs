use nox_mem::vec_types;

pub struct Iter<'a, Key, Val> {
    key_iter: vec_types::Iter<'a, Key>,
    val_iter: vec_types::Iter<'a, Val>,
}

impl <'a, Key, Val> Iter<'a, Key, Val> {

    pub unsafe fn new(key_iter: vec_types::Iter<'a, Key>, val_iter: vec_types::Iter<'a, Val>) -> Self {
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
    key_iter: vec_types::Iter<'a, Key>,
    val_iter: vec_types::IterMut<'a, Val>,
}

impl <'a, Key, Val> IterMut<'a, Key, Val> {

    pub unsafe fn new(key_iter: vec_types::Iter<'a, Key>, val_iter: vec_types::IterMut<'a, Val>) -> Self {
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
