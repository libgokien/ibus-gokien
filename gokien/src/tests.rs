use super::{transform_buffer, GokienEngine};

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
        ("ww", "ww"),
        ("www", "www"),
        ("wwawww", "wwawww"),
        ("waf", "ừa"),
        ("wwaos", "wwaos"),
        ("wwwwwa", "wwwwwa"),
        ("wwww", "wwww")
    ];
    for (word, expected) in cases.into_iter() {
        transform_buffer(word.chars(), &mut out);
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
        ("wwawww\n", "wwawww"),
        ("wwww\n", "wwww")
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
