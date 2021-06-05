pub mod key;
pub mod modifiers;

use std::iter::FromIterator;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::event::key::Key;
use crate::event::modifiers::Modifiers;

/// Represent key input from user.
///
/// KeyEvent should be immutable, so fields of this is private and have getter.
/// This is constructed by `Key` and `Modifiers`, some additional input like
/// "Shift", "Alt" and "Ctrl", which can merge with another one.
///
/// todo: Think C-SEALED for `Key` trait
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KeyEvent<K, M> {
  key: K,
  modifiers: M,
}

impl<K, M> KeyEvent<K, M> {
  /// Create new `KeyEvent`
  pub fn new(key: K, modifiers: M) -> Self {
    Self { key, modifiers }
  }
}

impl<K, M> KeyEvent<K, M>
where
  K: Key,
  M: Modifiers,
{
  /// Return key.
  ///
  /// todo: Consider changing to use reference.
  pub fn key(&self) -> K {
    self.key.clone()
  }

  /// Return modifiers.
  ///
  /// todo: Consider changing to use reference.
  pub fn modifiers(&self) -> M {
    self.modifiers.clone()
  }

  /// Check another key is included in self.
  /// if argument is same as, or has same key and its modifiers is subset of
  /// self's, return true, else return false.
  pub fn contains(&self, rhs: &Self) -> Option<M> {
    if self.key == rhs.key {
      Some(self.modifiers.clone() ^ rhs.modifiers.clone())
    } else {
      None
    }
  }

  /// Return Extended KeyEvent by Modifiers.
  pub fn or(self, rhs: M) -> Self {
    Self {
      modifiers: self.modifiers | rhs,
      ..self
    }
  }

  /// Return Extended KeyEvent by Modifiers.
  pub fn xor(self, rhs: M) -> Self {
    Self {
      modifiers: self.modifiers ^ rhs,
      ..self
    }
  }
}

impl<K: Copy, M: Copy> Copy for KeyEvent<K, M> {}

impl<K, M, I, Item> From<(K, I)> for KeyEvent<K, M>
where
  M: FromIterator<Item>,
  I: Iterator<Item = Item>,
  Item: Clone,
{
  fn from(value: (K, I)) -> Self {
    let (key, m) = value;
    Self {
      key,
      modifiers: m.collect(),
    }
  }
}
