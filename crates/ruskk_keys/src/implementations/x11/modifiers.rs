use crate::event::modifiers::Modifiers;
use bitflags::bitflags;

bitflags! {
    /// ModifierType is Flag represent the keys that modify input.
    /// repr `u32`
    #[derive(Default)]
    pub struct X11Modifier: u32 {
        const NONE = 0;
        const SHIFT_MASK= 1<<0;
        const LOCK_MASK= 1<<1;
        const CONTROL_MASK= 1<<2;
        const MOD1_MASK= 1<<3;
        const MOD2_MASK= 1<<4;
        const MOD3_MASK= 1<<5;
        const MOD4_MASK= 1<<6;
        const MOD5_MASK= 1<<7;

        // dummy modifiers for NICOLA
        // const LShiftMask = 1<<22;
        // const RShiftMask = 1<<23;
        // const UsleepMask = 1<<24;
        //
        // const SuperMask = 1<<26;
        // const HyperMask = 1<<27;
        const META_MASK= 1<<28;
        // const ReleaseMask = 1<<30;
    }
}

impl Modifiers for X11Modifier {
  fn is_empty(&self) -> bool {
    X11Modifier::is_empty(self)
  }
}
