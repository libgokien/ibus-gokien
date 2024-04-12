#[derive(Debug)]
enum State {
    Typing,
    PreeditCommitting,
    Interrupting,
    Backspacing,
}

use State::*;

#[derive(Debug)]
struct Buffer {
    buffer: Vec<char>,
    ouput: String,
}

fn process_inner(state: &mut State, buf: &mut Buffer, keyval: guint, key_state: guint) -> bool {
    if is_released(key_state) {
        return false;
    }
    match state {
        Typing => {
            if shift_shift(keyval) {
                *state = Interrupting;
                true
            } else if is_work_seps(keyval) {
                *state = PreeditCommitting;
                false
            } else if is_a_zA_Z(keyval) {
                buf.buffer.push(keyval);
                translate(buf.buffer, &mut buf.output);
                true
            } else if is_transparent(keyval) {
                false
            } else if is_backspace {
                state = Backspacing;
                true
            } else {
                *state = PreeditCommitting;
                false
            }
        }
        Interrupting => {
            if shift_shift(keyval) {
                *state = Typing;
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn process(state: &mut State, buf: &mut Buffer, keyval: guint, key_state: guint) -> bool {
    let processed = process_inner(state, buf, keyval, key_state);
    match state {
        Typing => update_preedit(),
        PreeditCommitting => {
            commit_preedit();
            buf.buffer.clear();
            buf.output.clear();
            *state = Typing;
        }
        Interrupting => commit_preedit(),
        Backspacing => {
            *state = Typing;
            return if bs_success(output) {
                update_preedit();
                true
            } else {
                false
            };
        }
    }
    processed
}

fn push_translate(buf: &mut Buffer) {
    unimplemented!()
}

fn translate(buffer: &[char], output: &mut String) {
    unimplemented!()
}

fn main() {
    let state = State::Typing;
    let buf = Buffer {
        buffer: Vec::new(),
        output: String::new(),
    };
    for (keyval, key_state) in receive_keyval() {
        process(&mut state, &mut buf, keyval, key_state);
    }
}

fn receive_keyval() -> impl Iterator<Item = ((keyval, key_state))> {
    unimplemented!()
}

fn commit_preedit() {
    unimplemented!()
}

fn update_preedit() {
    unimplemented!()
}
