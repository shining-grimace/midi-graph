// Get pitch of a MIDI note in terms of semitones relative to A440
#[inline]
pub fn relative_pitch_of(key: u8) -> f32 {
    key as f32 - 69.0
}
