use std::collections::hash_map::HashMap;

use core::hash::{Hash, BuildHasher};

pub trait ExtendExt<A> {

    fn try_extend<E, I>(&mut self, iter: I) -> Result<(), E>
        where I: IntoIterator<Item = Result<A, E>>;
}

impl<K, V, S> ExtendExt<(K, V)> for HashMap<K, V, S>
    where
        K: Eq + Hash,
        S: BuildHasher,
{

    fn try_extend<E, I>(&mut self, iter: I) -> Result<(), E>
        where I: IntoIterator<Item = Result<(K, V), E>>,
    {
        for pair in iter {
            let (k, v) = pair?;
            self.insert(k, v);
        }
        Ok(())
    }
}
