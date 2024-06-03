use vi::transform_buffer;

use super::GokienEngine;

#[test]
fn main() {
    tracing_subscriber::fmt::init();
    test_vi_whole_word();
    process_keys();
}

#[track_caller]
fn test_vi_whole_word() {
    let mut out = String::new();
    // (intput, expected)
    #[rustfmt::skip]
    let cases = [
        ("ww",      "w"),
        ("www",     "ww"),
        ("wwawww",  "wawww"),
        ("waf",     "ừa"),
        ("wwaos",   "waos"),
        ("wwwwwa",  "wwwwa"),
        ("wwww",    "www")
    ];
    for (word, expected) in cases.into_iter() {
        transform_buffer(&vi::TELEX, word.chars(), &mut out);
        assert_eq!(*expected, out);
        out.clear();
    }
}

#[track_caller]
fn process_keys() {
    let mut gokien = GokienEngine::new();
    // (intput, expected)
    #[rustfmt::skip]
    let cases = [
        ("nghieengx\n", "nghiễng"),
        ("wwawww\n",    "wawww"),
        ("wwww\n",      "www")
    ];
    for (word, expected) in cases.into_iter() {
        for ch in word.chars() {
            gokien.process_key(ch as u32, 0);
        }
        // debug!(?gokien.buffer);
        assert_eq!(*expected, gokien.output);
        gokien.clear();
        gokien.state = Default::default();
    }
}
