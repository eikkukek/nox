use core::ops::Deref;

use crate::generics::GenericIdents;

pub struct Input<'a> {
    input: &'a syn::DeriveInput,
    pub generic_idents: GenericIdents<'a>,
}

impl<'a> From<&'a syn::DeriveInput> for Input<'a> {

    fn from(value: &'a syn::DeriveInput) -> Self {
        Self {
            input: value,
            generic_idents: (&value.generics).into(),
        }
    }
}

impl<'a> Deref for Input<'a> {

    type Target = syn::DeriveInput;

    fn deref(&self) -> &Self::Target {
        self.input
    }
}
