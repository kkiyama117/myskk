pub mod key;
pub mod modifiers;

use std::fmt::{self, Display, Formatter};
use std::iter::FromIterator;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::event::key::Key;
use crate::event::modifiers::Modifiers;

/// todo: Think C-SEALED for `Key` trait
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KeyEvent<K, M> {
    key: K,
    modifiers: M,
}

impl<K, M> KeyEvent<K, M>
    where
        K: Key,
        M: Modifiers,
{
    pub fn new(key: K, modifiers: M) -> Self {
        Self { key, modifiers }
    }

    pub fn key(&self) -> K {
        self.key.clone()
    }

    pub fn modifiers(&self) -> M {
        self.modifiers.clone()
    }

    pub fn contains(&self, rhs: &Self) -> Option<M> {
        if self.key == rhs.key {
            Some(self.modifiers.clone() ^ rhs.modifiers.clone())
        } else {
            None
        }
    }

    pub fn extend_modifiers(&self, modifiers: M) -> Self {
        Self {
            key: self.key.clone(),
            modifiers: self.modifiers.clone() | modifiers,
        }
    }
}

impl<K: Copy, M: Copy> Copy for KeyEvent<K, M> {}

impl<K, M, Item> From<(K, Vec<Item>)> for KeyEvent<K, M>
    where
        K: Key,
        M: Modifiers + FromIterator<Item>,
        Item: Clone,
{
    fn from(value: (K, Vec<Item>)) -> Self {
        let (key, m) = value;
        Self {
            key,
            modifiers: m.iter().cloned().collect(),
        }
    }
}

impl<K: Display, M> Display for KeyEvent<K, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}
