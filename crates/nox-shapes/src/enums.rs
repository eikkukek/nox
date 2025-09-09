#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Winding {
    #[default]
    CounterClockWise = 0,
    ClockWise = 1,
}
