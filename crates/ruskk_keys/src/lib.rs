pub mod errors;
pub mod event;

#[cfg(feature = "b_tree")]
#[path = "implementations/b_tree.rs"]
pub mod b_tree;

#[cfg(feature = "x11")]
#[path = "implementations/x11/mod.rs"]
pub mod x11;

pub mod prelude {
  pub use crate::errors::KeyEventError;
  pub use crate::event::key::Key;
  pub use crate::event::modifiers::Modifiers;
  pub use crate::event::KeyEvent;
}
