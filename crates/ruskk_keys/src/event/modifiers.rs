use std::ops::{BitAnd, BitOr, BitXor};

/// Modifiers is key modifier
pub trait Modifiers:
  BitAnd<Output = Self>
  + BitOr<Output = Self>
  + BitXor<Output = Self>
  + Clone
  + Sized
{
  /// check if it is "empty"
  fn is_empty(&self) -> bool;
}
