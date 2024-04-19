#[cfg(test)]
mod tests;

use std::ffi::CString;
use std::mem;

use ribus::c::{self, guint};
use tracing::{debug, error};
use vi::telex::transform_buffer;

// account for incorrect typos
const MAX_CHARS_IN_WORD: usize = "nghieengz".len().next_power_of_two();
const EMPTY_STRING: String = String::new();

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum State {
    #[default]
    Typing,
    PreeditCommitting,
    Interrupting,
    Backspacing,
}

pub struct GokienEngine {
    buffer: Vec<char>,
    output: String,
    pub state: State,
}

impl GokienEngine {
    pub fn new() -> Self {
        Self {
            // FIXME: buffer and output only grows onwords, is that a problem?
            buffer: Vec::with_capacity(MAX_CHARS_IN_WORD),
            output: String::with_capacity(MAX_CHARS_IN_WORD),
            state: State::default(),
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.output.clear();
    }

    pub fn reset(&mut self) {
        self.clear();
        self.state = State::default();
    }

    pub fn process_key(&mut self, keyval: guint, state: guint) -> bool {
        use State::*;
        debug!(keyval, state);
        // do not handle key released events, only consider key pressed ones
        if state & c::IBUS_RELEASE_MASK != 0 {
            return false;
        }

        if state & c::IBUS_CONTROL_MASK != 0 || state & c::IBUS_MOD1_MASK != 0 {
            debug!("ignore Ctrl || Alt combo");
            if self.state == Typing {
                self.state = PreeditCommitting;
            }
            return false;
        }

        if keyval == c::IBUS_KEY_Shift_L || keyval == c::IBUS_KEY_Shift_R {
            if state & c::IBUS_SHIFT_MASK == 0 {
                debug!("ignore SHIFT key");
                return false;
            } else {
                debug!("SHIFT + SHIFT pressed");
                match self.state {
                    Typing => {
                        self.state = Interrupting;
                    }
                    Interrupting => {
                        self.state = Typing;
                    }
                    _ => {
                        unreachable!();
                    }
                }
                return true;
            }
        }

        match self.state {
            Typing => {
                match keyval {
                    c::IBUS_KEY_Caps_Lock..=c::IBUS_KEY_Hyper_R => {
                        debug!("ignore keys from Caps to Hyper");
                        false
                    }
                    c::IBUS_KEY_BackSpace => {
                        debug!("hande backspace");
                        self.state = Backspacing;
                        true
                    }
                    // telex only
                    c::IBUS_KEY_A..=c::IBUS_KEY_Z | c::IBUS_KEY_a..=c::IBUS_KEY_z | c::IBUS_KEY_0..=c::IBUS_KEY_9 => {
                        let ch = char::from(keyval as u8);
                        self.buffer.push(ch);
                        self.output.clear();
                        if let [w, head @ .., tail] = &*self.buffer {
                            if *w == 'w' && head.iter().all(|c| *c == 'w') {
                                self.output.extend(head);
                                self.output.push(*tail);
                                return true;
                            }
                        }
                        self.translate()
                    }
                    // non processed keys
                    _ => {
                        self.state = PreeditCommitting;
                        false
                    }
                }
            }
            _ => false,
        }
    }

    #[must_use]
    fn translate(&mut self) -> bool {
        if self.buffer.is_empty() {
            return false;
        }

        let buffer = self.buffer.iter().cloned();
        transform_buffer(buffer, &mut self.output);
        true
    }

    pub fn get_output(&self) -> &str {
        &*self.output
    }

    fn take_output(&mut self) -> String {
        mem::replace(&mut self.output, EMPTY_STRING)
    }

    pub fn take_output_as_cstr(&mut self) -> Option<CString> {
        if self.buffer.is_empty() {
            return None;
        }
        Some(cstr_from_str(self.take_output()))
    }

    fn replace_output(&mut self, s: String) {
        assert!(self.output.is_empty());
        let _ = mem::replace(&mut self.output, s);
    }

    pub fn replace_output_by_cstr(&mut self, s: CString) {
        let s = str_from_cstr(s);
        self.replace_output(s);
    }

    // Call this when backspace is pressed.
    // Returns false when our buffer is empty to return back
    // backspace for ibus.
    #[must_use]
    pub fn handle_backspace(&mut self) -> bool {
        debug!("handle_backspace");
        if self.buffer.is_empty() {
            return false;
        }
        let mut s = self.take_output();
        match s.pop() {
            Some(ch) => {
                debug!("backspaced char: {ch}");
                self.buffer.clear();
                self.buffer.extend(s.chars());
            }
            None => error!("s must not be empty"),
        }
        self.replace_output(s);
        true
    }
}

// ensure that `s` doesn't have internal NUL byte
pub fn cstr_from_str(s: String) -> CString {
    unsafe { CString::from_vec_unchecked(s.into_bytes()) }
}

pub fn str_from_cstr(s: CString) -> String {
    unsafe { String::from_utf8_unchecked(s.into_bytes()) }
}
