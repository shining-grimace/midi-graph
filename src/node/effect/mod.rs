
pub mod adsr;
pub mod fader;
pub mod lfo;
pub mod transition;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ModulationProperty {
    Volume,
    Pan,
    PitchMultiplier,
    MixBalance,
    TimeDilation
}

