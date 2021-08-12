// TODO-FIXME ALL THE VECTORS IN SHIPS SHOULD BE CHECKED
// Last meeting it turned out that somewhere in the code there's
// a possibility to `NaN` or `Inf` the vectors which causes the 
// computations to collapse. This must be fixed.

pub mod ship;
pub mod player;
pub mod render;
pub mod earth;
pub mod storage;
pub mod ai_machine;

pub use crate::earth::Earth;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PointerTarget {
    None,
    Sun,
    Earth,
}
