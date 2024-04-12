use super::GokienEngine;

#[test]
fn process_keys() {
    let mut gokien = GokienEngine::new();
    // (intput, expected)
    let cases: &[(&str, &str)] = &[
        ("nghieengx\n", "nghiễng"),
        // ("thanhs\n"),
        // ("albert\n"),
        // ("einstein\n)",
    ];
    for (word, expected) in cases.into_iter() {
        for ch in word.chars() {
            gokien.process_key(ch as u32, 0);
        }
        assert_eq!(*expected, gokien.output);
    }
}
