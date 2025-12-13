#![allow(unused_variables)]

use nox_mem::cell::CellToken;

use crate::*;

pub trait Initialize<Token: CellToken>: FnOnce(&mut Token, &mut win::WindowContext, &mut gpu::GpuContext) -> Result<()> {}

impl<Token: CellToken, F: FnOnce(&mut Token, &mut win::WindowContext, &mut gpu::GpuContext) -> Result<()>> Initialize<Token> for F {}

pub trait ProcessEvent<Token: CellToken>: FnMut(&mut Token, Event) -> Result<()> {}

impl<Token: CellToken, F: FnMut(&mut Token, Event) -> Result<()>> ProcessEvent<Token> for F {}
