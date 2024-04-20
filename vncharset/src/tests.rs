use super::VniVariant::*;
use super::{Tcvn3, Viqr, Viscii, Vni};

#[test]
fn main() {
    test_viqr();
    test_vni();

    let unicode = "Hàn Quốc hôm nay bắn pháo đáp trả Triều Tiên, bởi cho rằng pháo của Triều Tiên bắn sang vùng biển của nước này ";
    let vni_win = b"Ha\xf8n Quo\xE1c ho\xE2m nay ba\xE9n pha\xf9o \xf1a\xf9p tra\xfb Trie\xe0u Tie\xE2n, b\xf4\xfbi cho ra\xe8ng pha\xf9o cu\xfba Trie\xe0u Tie\xE2n ba\xE9n sang vu\xf8ng bie\xe5n cu\xfba n\xf6\xf4\xf9c na\xf8y ";
    let viqr    = b"Ha`n Quo^'c ho^m nay ba('n pha'o dda'p tra? Trie^`u Tie^n, bo+?i cho ra(`ng pha'o cu?a Trie^`u Tie^n ba('n sang vu`ng bie^?n cu?a nu+o+'c na`y ";
    let tcvn3   = b"H\xb5n Qu\xe8c h\xabm nay b\xben ph\xb8o \xae\xb8p tr\xb6 Tri\xd2u Ti\xaan, b\xebi cho r\xbbng ph\xb8o c\xf1a Tri\xd2u Ti\xaan b\xben sang v\xefng bi\xd3n c\xf1a n\xad\xedc n\xb5y ";

    assert_eq!(Vni::from_bytes(vni_win).encode_utf8(AnsiWin), unicode);
    assert_eq!(Viqr::from_bytes(viqr).encode_utf8(), unicode);
    assert_eq!(Tcvn3::from_bytes(tcvn3).encode_utf8(), unicode);

    let viscii = b"H\xe0n Qu\xafc h\xf4m nay";
    let unicode = "Hàn Quốc hôm nay";
    assert_eq!(Viscii::from_bytes(viscii).encode_utf8(), unicode);
}

#[track_caller]
fn test_vni() {
    let vni_doc = b"T\x93i y\x88u ti\x89ng n\x9f\xf4c t\x93i t\xa5 khi m\xf4i ra \xad\xf5i.";
    let expected = "Tôi yêu tiếng nước tôi từ khi mới ra đời.";
    assert_eq!(Vni::from_bytes(vni_doc).encode_utf8(MsDoc), expected);
}

#[track_caller]
fn test_viqr() {
    let cases = [
        (
            &br"To^i ye^u tie^'ng nu+o+'c to^i tu+` khi mo+'i ra ddo+`i\. "[..],
            "Tôi yêu tiếng nước tôi từ khi mới ra đời. ",
        ),
        (
            &br"O^ng te^n gi`\? To^i te^n la` Tra^`n Va(n Hie^'u\."[..],
            "Ông tên gì? Tôi tên là Trần Văn Hiếu.",
        ),
        (&br"A+ "[..], "A+ "),
    ];
    for (input, expected) in cases[2..].iter() {
        assert_eq!(Viqr::from_bytes(input).encode_utf8(), *expected);
    }
}
