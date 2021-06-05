use crate::errors::KeyEventParseFailedError;
use crate::event::key::Key;
use crate::event::modifiers::Modifiers;
use crate::event::KeyEvent;
use crate::x11::modifiers::X11Modifier;
use crate::x11::XKeySymbol;
use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;
use std::convert::TryFrom;
use std::fmt::Display;
use std::marker::PhantomData;
use std::option::Option::Some;
use std::str::Chars;

#[derive(Clone, Debug)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct CompatibleParser<'a, K, M> {
  iter: Chars<'a>,
  position: usize,
  result_meta: PhantomData<fn() -> (K, M)>,
}

impl<'a, K, M> CompatibleParser<'a, K, M>
where
  K: Key + TryFrom<InputStrType>,
  M: Modifiers + TryFrom<InputStrType>,
  <K as TryFrom<InputStrType>>::Error:
    From<<M as TryFrom<InputStrType>>::Error>,
  KeyEventParseFailedError: From<<K as TryFrom<InputStrType>>::Error>,
{
  /// get next keyEvent from String and rule with simple state machine
  pub fn parse_next(
    &mut self,
  ) -> Option<Result<KeyEvent<K, M>, KeyEventParseFailedError>> {
    let mut is_escaped = false;
    let input = self
      .iter
      .clone()
      .skip(self.position)
      .fold_while(InputStrType::None, |accum, c| {
        self.position += 1;
        if is_escaped {
          match accum {
            InputStrType::Normal(s) => {
              Continue(InputStrType::Normal(format!("{}{}", s, c)))
            }
            InputStrType::Special(s) => {
              Continue(InputStrType::Special(format!("{}{}", s, c)))
            }
            InputStrType::Error(e) => Done(InputStrType::Error(e)),
            InputStrType::None => {
              Continue(InputStrType::Normal(String::from(c)))
            }
          }
        } else {
          match c {
            ' ' => Done(accum),
            '\\' => {
              is_escaped = true;
              Continue(accum)
            }
            _ => match accum {
              InputStrType::Normal(s) => match c {
                '(' => Continue(InputStrType::Special(String::new())),
                ')' => Done(InputStrType::Error(KeyEventParseFailedError {
                  msg: "bare ')' is not allowed in complex keyseq".to_string(),
                })),
                _ => Continue(InputStrType::Normal(format!("{}{}", s, c))),
              },
              InputStrType::Special(s) => match c {
                '(' => Done(InputStrType::Error(KeyEventParseFailedError {
                  msg: "bare '(' is not allowed in complex keyseq".to_string(),
                })),
                ')' => Continue(InputStrType::Special(s)),
                _ => Continue(InputStrType::Special(format!("{}{}", s, c))),
              },
              InputStrType::Error(e) => Done(InputStrType::Error(e)),
              InputStrType::None => {
                Continue(InputStrType::Normal(String::from(c)))
              }
            },
          }
        }
      })
      .into_inner();
    match input {
      InputStrType::None => None,
      InputStrType::Normal(_) | InputStrType::Special(_) => Some(
        K::try_from(input.clone())
          .and_then(|key| {
            M::try_from(input.clone())
              .map(|modifier| KeyEvent::new(key, modifier))
              .map_err(|e| e.into())
          })
          .map_err(|e| e.into()),
      ),
      InputStrType::Error(e) => Some(Err(e)),
    }
  }
}

impl<'a, K, M> Iterator for CompatibleParser<'a, K, M>
where
  K: Key + TryFrom<InputStrType>,
  M: Modifiers + TryFrom<InputStrType>,
  <K as TryFrom<InputStrType>>::Error:
    From<<M as TryFrom<InputStrType>>::Error>,
  KeyEventParseFailedError: From<<K as TryFrom<InputStrType>>::Error>,
{
  type Item = Result<KeyEvent<K, M>, KeyEventParseFailedError>;
  fn next(&mut self) -> Option<Self::Item> {
    self.parse_next()
  }
}

impl<'a, K, M> Display for CompatibleParser<'a, K, M> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.iter.as_str())
  }
}

#[derive(Clone, Debug)]
pub enum InputStrType {
  None,
  Normal(String),
  Special(String),
  Error(KeyEventParseFailedError),
}

impl TryFrom<InputStrType> for XKeySymbol {
  type Error = KeyEventParseFailedError;

  fn try_from(value: InputStrType) -> Result<Self, Self::Error> {
    match value {
      InputStrType::Error(e) => Err(e),
      InputStrType::Normal(x) => x11_keysymdef::lookup_by_name(x.as_str())
        .map(|x| x.unicode)
        .ok_or(KeyEventParseFailedError { msg: x.to_string() }),
      InputStrType::Special(x) => x11_keysymdef::lookup_by_name(x.as_str())
        .map(|x| x.unicode)
        .ok_or(KeyEventParseFailedError { msg: x.to_string() }),
      InputStrType::None => Err(KeyEventParseFailedError {
        msg: "void".to_string(),
      }),
    }
  }
}

impl TryFrom<InputStrType> for X11Modifier {
  type Error = KeyEventParseFailedError;

  fn try_from(value: InputStrType) -> Result<Self, Self::Error> {
    match value {
      InputStrType::Normal(base) => base
        .split('-')
        .collect_vec()
        .split_last()
        .ok_or(KeyEventParseFailedError {
          msg: format!("unknown modifier"),
        })?
        .1
        .iter()
        .try_fold(X11Modifier::NONE, |accum, x| match *x {
          "S" => Ok(accum | X11Modifier::SHIFT_MASK),
          "C" => Ok(accum | X11Modifier::CONTROL_MASK),
          "M" => Ok(accum | X11Modifier::META_MASK),
          "A" => Ok(accum | X11Modifier::MOD1_MASK),
          "G" => Ok(accum | X11Modifier::MOD5_MASK),
          _ => Err(KeyEventParseFailedError {
            msg: format!("unknown modifier {}", x),
          }),
        }),
      InputStrType::Special(base) => base
        .split_whitespace()
        .collect_vec()
        .split_last()
        .ok_or(KeyEventParseFailedError {
          msg: format!("unknown modifier"),
        })?
        .1
        .iter()
        .try_fold(X11Modifier::NONE, |accum, x| match *x {
          "shift" => Ok(accum | X11Modifier::SHIFT_MASK),
          "control" => Ok(accum | X11Modifier::CONTROL_MASK),
          "alt" => Ok(accum | X11Modifier::MOD1_MASK),
          _ => Err(KeyEventParseFailedError {
            msg: format!("unknown modifier {}", x),
          }),
        }),
      InputStrType::Error(e) => Err(e),
      _ => Err(KeyEventParseFailedError {
        msg: "void".to_string(),
      }),
    }
  }
}

pub trait StringExt {
  fn key_events<K, M>(&self) -> CompatibleParser<'_, K, M>;
}

impl StringExt for String {
  fn key_events<K, M>(&self) -> CompatibleParser<'_, K, M> {
    CompatibleParser {
      iter: self.chars(),
      position: 0,
      result_meta: Default::default(),
    }
  }
}
impl<'a> StringExt for &'a str {
  fn key_events<K, M>(&self) -> CompatibleParser<'_, K, M> {
    CompatibleParser {
      iter: self.chars(),
      position: 0,
      result_meta: Default::default(),
    }
  }
}
