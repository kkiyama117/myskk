pub mod compatible;
mod modifiers;

type XKeySymbol = char;
impl crate::event::key::Key for XKeySymbol {}

pub type KeyEvent = crate::event::KeyEvent<XKeySymbol, modifiers::X11Modifier>;
pub type Key = XKeySymbol;
pub type Modifiers = modifiers::X11Modifier;
