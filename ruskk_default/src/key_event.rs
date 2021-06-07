use itertools::{
  FoldWhile::{Continue, Done},
  Itertools,
};
use ruskk_keys::{
  prelude::{Key, KeyEvent, KeyEventError, Modifiers},
  x11::{Key as X11Key, Modifiers as X11Modifier},
};
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::str::Chars;

#[derive(Clone, Debug)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct CompatibleParser<'a, K, M> {
  iter: Chars<'a>,
  position: usize,
  result_meta: PhantomData<fn() -> (K, M)>,
}

impl<'a, K, M> Iterator for CompatibleParser<'a, K, M>
where
  K: Key + TryFrom<InputStrType, Error = KeyEventError>,
  M: Modifiers + TryFrom<InputStrType, Error = KeyEventError>,
{
  type Item = Result<KeyEvent<K, M>, KeyEventError>;
  fn next(&mut self) -> Option<Self::Item> {
    self.parse_next()
  }
}

impl<'a, K, M, E> CompatibleParser<'a, K, M>
where
  K: Key + TryFrom<InputStrType, Error = E>,
  M: Modifiers + TryFrom<InputStrType, Error = E>,
  E: Into<KeyEventError>,
{
  pub fn parse_text(&mut self) -> Result<InputStrType, KeyEventError> {
    let mut is_escaped = false;
    self
      .iter
      .clone()
      .skip(self.position)
      .fold_while(Ok(InputStrType::None), |accum, c| {
        self.position += 1;
        if let Ok(accum) = accum {
          if is_escaped {
            match accum {
              InputStrType::Normal(s) => {
                Continue(Ok(InputStrType::Normal(format!("{}{}", s, c))))
              }
              InputStrType::Special(s) => {
                Continue(Ok(InputStrType::Special(format!("{}{}", s, c))))
              }
              InputStrType::None => {
                Continue(Ok(InputStrType::Normal(String::from(c))))
              }
            }
          } else {
            match c {
              ' ' => Done(Ok(accum)),
              '\\' => {
                is_escaped = true;
                Continue(Ok(accum))
              }
              _ => match accum {
                InputStrType::Normal(s) => match c {
                  '(' => Continue(Ok(InputStrType::Special(String::new()))),
                  ')' => Done(Err(KeyEventError::ParseFailed {
                    msg: "bare ')' is not allowed in complex keyseq"
                      .to_string(),
                  })),
                  _ => {
                    Continue(Ok(InputStrType::Normal(format!("{}{}", s, c))))
                  }
                },
                InputStrType::Special(s) => match c {
                  '(' => Done(Err(KeyEventError::ParseFailed {
                    msg: "bare '(' is not allowed in complex keyseq"
                      .to_string(),
                  })),
                  ')' => Continue(Ok(InputStrType::Special(s))),
                  _ => {
                    Continue(Ok(InputStrType::Special(format!("{}{}", s, c))))
                  }
                },
                InputStrType::None => match c {
                  '(' => Continue(Ok(InputStrType::Special(String::new()))),
                  ')' => Done(Err(KeyEventError::ParseFailed {
                    msg: "bare ')' is not allowed in complex keyseq"
                      .to_string(),
                  })),
                  _ => Continue(Ok(InputStrType::Normal(format!("{}", c)))),
                },
              },
            }
          }
        } else {
          // accum must not be error if continued, hence return error as it is.
          Done(accum)
        }
      })
      .into_inner()
  }
  /// get next keyEvent from String and rule with simple state machine
  pub fn parse_next(
    &mut self,
  ) -> Option<Result<KeyEvent<K, M>, KeyEventError>> {
    match self.parse_text() {
      Ok(input) => match input {
        InputStrType::None => None,
        InputStrType::Normal(_) | InputStrType::Special(_) => Some(
          K::try_from(input.clone())
            .and_then(|key| {
              M::try_from(input.clone())
                .map(|modifier| KeyEvent::new(key, modifier))
            })
            .map_err(|e| e.into()),
        ),
      },
      Err(e) => Some(Err(e.into())),
    }
  }
}
impl TryFrom<InputStrType> for X11Key {
  type Error = KeyEventError;

  fn try_from(value: InputStrType) -> Result<Self, Self::Error> {
    match value {
      InputStrType::Normal(x) => x
        .split('-')
        .last()
        .and_then(|x| x11_keysymdef::lookup_by_name(x).map(|x| x.unicode))
        .ok_or(Self::Error::ParseFailed {
          msg: format!("unknown key: {}", x),
        }),
      InputStrType::Special(x) => Ok(
        x11_keysymdef::lookup_by_name(x.as_str())
          .map(|x| x.unicode)
          .unwrap_or('\0'),
      ),
      InputStrType::None => Err(Self::Error::ParseFailed {
        msg: "void".to_string(),
      }),
    }
  }
}

impl TryFrom<InputStrType> for X11Modifier {
  type Error = KeyEventError;

  fn try_from(value: InputStrType) -> Result<Self, Self::Error> {
    match value {
      InputStrType::Normal(base) => base
        .split('-')
        .collect_vec()
        .split_last()
        .ok_or(KeyEventError::ParseFailed {
          msg: format!("unknown modifier: {}", base),
        })?
        .1
        .iter()
        .try_fold(X11Modifier::NONE, |accum, x| match *x {
          // support only limited modifiers in this form
          "S" => Ok(accum | X11Modifier::SHIFT_MASK),
          "C" => Ok(accum | X11Modifier::CONTROL_MASK),
          "M" => Ok(accum | X11Modifier::META_MASK),
          "A" => Ok(accum | X11Modifier::MOD1_MASK),
          "G" => Ok(accum | X11Modifier::MOD5_MASK),
          _ => Err(KeyEventError::ParseFailed {
            msg: format!("unknown modifier: {}", x),
          }),
        }),
      InputStrType::Special(base) => base.split_whitespace().try_fold(
        X11Modifier::NONE,
        |accum, x| match x {
          "shift" => Ok(accum | X11Modifier::SHIFT_MASK),
          "control" => Ok(accum | X11Modifier::CONTROL_MASK),
          "alt" => Ok(accum | X11Modifier::MOD1_MASK),
          _ => Err(KeyEventError::ParseFailed {
            msg: format!("unknown modifier: {}", x),
          }),
        },
      ),
      InputStrType::None => Err(Self::Error::ParseFailed {
        msg: "void key".to_string(),
      }),
    }
  }
}

#[derive(Clone, Debug)]
pub enum InputStrType {
  None,
  Normal(String),
  Special(String),
}

pub trait StringExt {
  fn as_key_events<K, M>(&self) -> CompatibleParser<'_, K, M>;
}

impl StringExt for String {
  fn as_key_events<K, M>(&self) -> CompatibleParser<'_, K, M> {
    CompatibleParser {
      iter: self.chars(),
      position: 0,
      result_meta: Default::default(),
    }
  }
}

impl<'a> StringExt for &'a str {
  fn as_key_events<K, M>(&self) -> CompatibleParser<'_, K, M> {
    CompatibleParser {
      iter: self.chars(),
      position: 0,
      result_meta: Default::default(),
    }
  }
}
