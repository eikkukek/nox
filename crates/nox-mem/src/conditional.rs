pub trait Conditional: 'static + Copy {

    const VALUE: bool;
}

#[derive(Clone, Copy)]
pub struct True;

impl Conditional for True {

    const VALUE: bool = true;
}

#[derive(Clone, Copy)]
pub struct False;

impl Conditional for False {

    const VALUE: bool = false;
}
