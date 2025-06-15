use crate::Balance;

pub const fn none_id() -> Option<u64> {
    None
}

pub const fn soundfont_polyphony_voices() -> usize {
    4
}

pub const fn amplitude() -> f32 {
    0.5
}

pub const fn duty_cycle() -> f32 {
    0.5
}

pub const fn note_for_16_shifts() -> u8 {
    64
}

pub const fn attack() -> f32 {
    0.125
}

pub const fn decay() -> f32 {
    0.25
}

pub const fn sustain() -> f32 {
    0.5
}

pub const fn release() -> f32 {
    0.125
}

pub const fn source_balance() -> Balance {
    Balance::Both
}

pub const fn mixer_balance() -> f32 {
    0.5
}

pub const fn max_voices() -> usize {
    4
}

