use vi::processor::{LetterModification, ToneMark};

pub fn maybe_tone_mark(b: u8) -> (Option<ToneMark>, Option<LetterModification>) {
    use LetterModification::*;
    use ToneMark::*;
    match b {
        0xE0 | 0xC0 => (Some(Grave), Some(Circumflex)),
        0xE1 | 0xC1 => (Some(Acute), Some(Circumflex)),
        0xE2 | 0xC2 => (None, Some(Circumflex)),
        0xE3 | 0xC3 => (Some(Tilde), Some(Circumflex)),
        0xE4 | 0xC4 => (Some(Underdot), Some(Circumflex)),
        0xE5 | 0xC5 => (Some(HookAbove), Some(Circumflex)),
        //
        0xE8 | 0xC8 => (Some(Grave), Some(Breve)),
        0xE9 | 0xC9 => (Some(Acute), Some(Breve)),
        0xEA | 0xCA => (None, Some(Breve)),
        0xEB | 0xCB => (Some(Underdot), Some(Breve)),
        //
        0xEF | 0xCF => (Some(Underdot), None),
        0xF5 | 0xD5 => (Some(Tilde), None),
        0xF8 | 0xD8 => (Some(Grave), None),
        0xF9 | 0xD9 => (Some(Acute), None),
        0xFA | 0xDA => (Some(HookAbove), Some(Circumflex)),
        0xFB | 0xDB => (Some(HookAbove), None),
        0xFC | 0xDC => (Some(Tilde), Some(Circumflex)),
        _ => (None, None),
    }
}
