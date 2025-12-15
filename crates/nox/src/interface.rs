#![allow(unused_variables)]

use crate::*;

pub trait Interface: FnMut(Event) -> Result<()> {}

impl<F: FnMut(Event) -> Result<()>> Interface for F {}
