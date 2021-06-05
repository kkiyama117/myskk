use std::ops::{BitAnd, BitOr, BitXor};

/// Modifiers is key modifier
pub trait Modifiers:
    BitAnd<Output = Self> + BitOr<Output = Self> + BitXor<Output = Self> + Clone + Sized
{
    fn is_empty(&self) -> bool;
}
