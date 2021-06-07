use itertools::{
  FoldWhile::{Continue, Done},
  Itertools,
};
use ruskk_keys::{
  prelude::{KeyEvent, KeyEventError},
  x11::{Key as X11Key, Modifiers as X11Modifier},
};
use smol_str::SmolStr;
// use std::marker::PhantomData;
use std::str::Chars;

#[derive(Clone, Debug)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct CompatibleParser<'a> {
  iter: Chars<'a>,
  position: usize,
  // result_meta: PhantomData<fn() -> (K, M)>,
}

impl<'a> Iterator for CompatibleParser<'a> {
  type Item = Result<KeyEvent<X11Key, X11Modifier>, KeyEventError>;
  fn next(&mut self) -> Option<Self::Item> {
    match self.parse_text() {
      Ok(x) => Some(extract_key_event(x)),
      Err(KeyEventError::Done) => None,
      Err(e) => Some(Err(e)),
    }
  }
}

impl<'a> CompatibleParser<'a> {
  fn parse_text(&mut self) -> Result<InputTokenStruct, KeyEventError> {
    let mut is_escaped = false;
    self
      .iter
      .clone()
      .skip(self.position)
      .fold_while(
        Ok(InputTokenStructInner {
          kind: InputStrType::Normal,
          raw: Default::default(),
        }),
        |accum, c| {
          self.position += 1;
          if let Ok(InputTokenStructInner { raw, kind }) = accum {
            if is_escaped {
              is_escaped = false;
              Continue(Ok(InputTokenStructInner {
                kind,
                raw: format!("{}{}", raw, c),
              }))
            } else {
              match kind {
                InputStrType::Normal => match c {
                  '\\' => {
                    is_escaped = true;
                    Continue(Ok(InputTokenStructInner { kind, raw }))
                  }
                  ' ' => Done(Ok(InputTokenStructInner { kind, raw })),
                  '(' => Continue(Ok(InputTokenStructInner {
                    kind: InputStrType::Special,
                    raw: Default::default(),
                  })),
                  ')' => Done(Err(KeyEventError::ParseFailed {
                    msg: "bare ')' is not allowed in complex keyseq"
                      .to_string(),
                  })),
                  _ => {
                    if self.position >= self.iter.clone().count() - 1 {
                      Done(Ok(InputTokenStructInner {
                        kind,
                        raw: format!("{}{}", raw, c),
                      }))
                    } else {
                      Continue(Ok(InputTokenStructInner {
                        kind,
                        raw: format!("{}{}", raw, c),
                      }))
                    }
                  }
                },
                InputStrType::Special => match c {
                  '\\' => {
                    is_escaped = true;
                    Continue(Ok(InputTokenStructInner { kind, raw }))
                  }
                  ' ' => Done(Ok(InputTokenStructInner { kind, raw })),
                  '(' => Done(Err(KeyEventError::ParseFailed {
                    msg: "bare '(' is not allowed in complex keyseq"
                      .to_string(),
                  })),
                  ')' => Continue(Ok(InputTokenStructInner {
                    kind: InputStrType::Special,
                    raw: Default::default(),
                  })),
                  _ => {
                    if self.position >= self.iter.clone().count() - 1 {
                      Done(Ok(InputTokenStructInner {
                        kind,
                        raw: format!("{}{}", raw, c),
                      }))
                    } else {
                      Continue(Ok(InputTokenStructInner {
                        kind,
                        raw: format!("{}{}", raw, c),
                      }))
                    }
                  }
                },
              }
            }
          } else {
            Done(accum)
          }
        },
      )
      .into_inner()
      .map(|InputTokenStructInner { raw, kind }| InputTokenStruct {
        kind,
        raw: SmolStr::from(raw),
      })
  }
}

fn extract_key_event(
  value: InputTokenStruct,
) -> Result<KeyEvent<X11Key, X11Modifier>, KeyEventError> {
  match value.kind() {
    InputStrType::Normal => {
      if let Some((last, other)) =
        value.as_str().split('-').collect_vec().split_last()
      {
        let key = extract_key_normal(last)?;
        let modifiers = extract_modifiers_normal(other)?;
        Ok(KeyEvent::new(key, modifiers))
      } else {
        Ok(KeyEvent::new(
          extract_key_normal(value.as_str())?,
          X11Modifier::NONE,
        ))
      }
    }
    InputStrType::Special => {
      if let Some((last, other)) =
        value.as_str().split_whitespace().collect_vec().split_last()
      {
        let key = extract_key_special(last)?;
        let modifiers = extract_modifiers_special(other)?;
        Ok(KeyEvent::new(key, modifiers))
      } else {
        Ok(KeyEvent::new(
          extract_key_special(value.as_str())?,
          X11Modifier::NONE,
        ))
      }
    }
  }
}

fn extract_key_normal(x: &str) -> Result<X11Key, KeyEventError> {
  x11_keysymdef::lookup_by_name(x).map(|x| x.unicode).ok_or(
    KeyEventError::ParseFailed {
      msg: format!("unknown key: {}", x),
    },
  )
}

fn extract_key_special(x: &str) -> Result<X11Key, KeyEventError> {
  Ok(
    x11_keysymdef::lookup_by_name(x)
      .map(|x| x.unicode)
      .unwrap_or('\0'),
  )
}

fn extract_modifiers_normal(v: &[&str]) -> Result<X11Modifier, KeyEventError> {
  v.iter().try_fold(X11Modifier::NONE, |accum, x| match *x {
    // support only limited modifiers in this form
    "S" => Ok(accum | X11Modifier::SHIFT_MASK),
    "C" => Ok(accum | X11Modifier::CONTROL_MASK),
    "M" => Ok(accum | X11Modifier::META_MASK),
    "A" => Ok(accum | X11Modifier::MOD1_MASK),
    "G" => Ok(accum | X11Modifier::MOD5_MASK),
    _ => Err(KeyEventError::ParseFailed {
      msg: format!("unknown modifier: {}", x),
    }),
  })
}

fn extract_modifiers_special(v: &[&str]) -> Result<X11Modifier, KeyEventError> {
  v.iter().try_fold(X11Modifier::NONE, |accum, x| match *x {
    "shift" => Ok(accum | X11Modifier::SHIFT_MASK),
    "control" => Ok(accum | X11Modifier::CONTROL_MASK),
    "alt" => Ok(accum | X11Modifier::MOD1_MASK),
    _ => Err(KeyEventError::ParseFailed {
      msg: format!("unknown modifier: {}", x),
    }),
  })
}

trait InputToken {
  fn kind(&self) -> InputStrType;
  fn as_str(&self) -> &str;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum InputStrType {
  Normal,
  Special,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InputTokenStruct {
  kind: InputStrType,
  raw: SmolStr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InputTokenStructInner {
  kind: InputStrType,
  raw: String,
}

impl InputToken for InputTokenStruct {
  fn kind(&self) -> InputStrType {
    self.kind.clone()
  }

  fn as_str(&self) -> &str {
    self.raw.as_str()
  }
}

pub trait StringExt {
  fn as_key_events(&self) -> CompatibleParser<'_>;
}

impl StringExt for String {
  fn as_key_events(&self) -> CompatibleParser<'_> {
    CompatibleParser {
      iter: self.chars(),
      position: 0,
      // result_meta: Default::default(),
    }
  }
}

impl<'a> StringExt for &'a str {
  fn as_key_events(&self) -> CompatibleParser<'_> {
    CompatibleParser {
      iter: self.chars(),
      position: 0,
      // result_meta: Default::default(),
    }
  }
}
