/// Main element of `KeyEvent`.
/// This trait should implement `Eq` ,`Clone`
pub trait Key: PartialEq + Eq + std::fmt::Debug + Clone{}

impl<'a> Key for &'a str {}
impl Key for String {}
impl Key for usize {}
impl Key for u8 {}
impl Key for u16 {}
impl Key for u32 {}
impl Key for u64 {}
impl Key for isize {}
impl Key for i8 {}
impl Key for i16 {}
impl Key for i32 {}
impl Key for i64 {}
