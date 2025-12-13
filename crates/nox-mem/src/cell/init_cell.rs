use core::{
    cell::UnsafeCell,
};

use super::CellToken;

struct Cell<T> {
    data: Option<T>,
}

pub struct InitCell<T, Token: CellToken>
{
    cell: UnsafeCell<Cell<T>>,
    identifier: Token::Identifier,
}

impl<T, Token: CellToken> InitCell<T, Token>
{

    #[inline(always)]
    pub fn new(token: &mut Token) -> Self
    {
        Self {
            cell: UnsafeCell::new(Cell {
                data: None,
            }),
            identifier: token.identifier(),
        }
    }

    #[inline(always)]
    pub fn get_or_init(
        &self,
        token: &mut Token,
        f: impl FnOnce() -> T,
    ) -> &mut T
    {
        token.validate(&self.identifier);
        unsafe {
            if let Some(data) = &mut (&mut *self.cell.get()).data {
                data
            } else {
                (&mut *self.cell.get()).data.insert(f())
            }
        }
    }
   
    #[inline(always)]
    pub fn get_or_try_init<E>(
        &self,
        token: &mut Token,
        f: impl FnOnce() -> Result<T, E>,
    ) -> Result<&mut T, E>
    {
        token.validate(&self.identifier);
        unsafe {
            if let Some(data) = &mut (&mut *self.cell.get()).data {
                Ok(data)
            } else {
                Ok((&mut *self.cell.get()).data.insert(f()?))
            }
        }
    }

    #[inline(always)]
    pub fn borrow(&self, token: &Token) -> Option<&T> {
        token.validate(&self.identifier);
        let cell = unsafe { &*self.cell.get() };
        cell.data.as_ref()
    }

    #[inline(always)]
    pub fn borrow_mut(&self, token: &mut Token) -> Option<&mut T> {
        token.validate(&self.identifier);
        let cell = unsafe { &mut *self.cell.get() };
        cell.data.as_mut()
    }

    #[inline(always)]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        let cell = self.cell.get_mut();
        cell.data.as_mut()
    }
}
