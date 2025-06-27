pub trait Conditional {}

pub struct True {}

impl Conditional for True {}

pub struct False {}

impl Conditional for False {}
