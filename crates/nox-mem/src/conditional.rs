pub trait Conditional {

    fn value() -> bool;
}

pub struct True {}

impl Conditional for True {

    fn value() -> bool {
        true
    }
}

pub struct False {}

impl Conditional for False {

    fn value() -> bool {
        false
    }
}
