use crate::event::modifiers::Modifiers;
use std::collections::BTreeSet;
use std::iter::FromIterator;
use std::ops::{BitAnd, BitOr, BitXor};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Example implementation for `ModifiersTrait` trait
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BTreeModifiers<M: Ord + Clone> {
  value: BTreeSet<M>,
}

impl<M: Ord + Clone> BitAnd<BTreeModifiers<M>> for BTreeModifiers<M> {
  type Output = BTreeModifiers<M>;

  fn bitand(self, rhs: BTreeModifiers<M>) -> Self::Output {
    Self {
      value: &self.value & &rhs.value,
    }
  }
}

impl<M: Ord + Clone> BitOr<BTreeModifiers<M>> for BTreeModifiers<M> {
  type Output = BTreeModifiers<M>;

  fn bitor(self, rhs: BTreeModifiers<M>) -> Self::Output {
    Self {
      value: &self.value | &rhs.value,
    }
  }
}

impl<M: Ord + Clone> BitXor<BTreeModifiers<M>> for BTreeModifiers<M> {
  type Output = BTreeModifiers<M>;

  fn bitxor(self, rhs: BTreeModifiers<M>) -> Self::Output {
    Self {
      value: &self.value ^ &rhs.value,
    }
  }
}

impl<I: Ord + Clone> FromIterator<I> for BTreeModifiers<I> {
  fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
    Self {
      value: BTreeSet::from_iter(iter),
    }
  }
}

impl<M: Ord + Clone> Modifiers for BTreeModifiers<M> {
  fn is_empty(&self) -> bool {
    self.value.is_empty()
  }
}

impl<I: Ord + Clone> From<Vec<I>> for BTreeModifiers<I> {
  fn from(v: Vec<I>) -> Self {
    v.into_iter().collect()
  }
}

#[cfg(test)]
mod tests {
  use crate::b_tree::BTreeModifiers;
  use crate::event::modifiers::Modifiers;
  use crate::event::KeyEvent;

  type BKeyEvent<'a> = KeyEvent<&'a str, BTreeModifiers<&'a str>>;

  #[test]
  fn matching_eq() {
    let a = BKeyEvent::new("foo", vec!["shift"].into());
    let b = BKeyEvent::new("foo", vec!["shift"].into());
    assert_eq!(a, b);
    assert_eq!(a.contains(&b), Some(BTreeModifiers::default()));
    assert!(!a.modifiers().is_empty());
  }

  #[test]
  fn subset_eq() {
    let a = BKeyEvent::new("foo", vec!["shift", "alt"].into());
    let b = BKeyEvent::new("foo", vec!["shift"].into());
    assert_eq!(a.contains(&b), Some(vec!["alt"].into_iter().collect()));
  }

  #[test]
  fn subset_not_eq() {
    let a = BKeyEvent::new("foo", vec!["shift"].into());
    let b = BKeyEvent::new("bar", vec!["shift"].into());
    assert_eq!(a.contains(&b), None);
  }
}
