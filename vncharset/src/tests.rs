use super::{Tcvn3, Viqr, Viscii};

#[test]
fn main() {
    test_viqr();

    let unicode = "Hàn Quốc hôm nay bắn pháo đáp trả Triều Tiên, bởi cho rằng pháo của Triều Tiên bắn sang vùng biển của nước này";
    // let vni     = b"Ha\xb8n Quo\xa1c ho\xa2m nay ba\xa9n pha\xb9o \xb1a\xb9p tra\xbb Trie\xa0u Tie\xa2n, b\xb4\xbbi cho ra\xa8ng pha\xb9o cu\xbba Trie\xa0u Tie\xa2n ba\xa9n sang vu\xb8ng bie\xa5n cu\xbba n\xb6\xb4\xb9c na\xb8y";
    // let viqr    = b"Ha`n Quo^'c ho^m nay ba('n pha'o dda'p tra? Trie^`u Tie^n, bo+?i cho ra(`ng pha'o cu?a Trie^`u Tie^n ba('n sang vu`ng bie^?n cu?a nu+o+'c na`y";
    let tcvn3   = b"H\xb5n Qu\xe8c h\xabm nay b\xben ph\xb8o \xae\xb8p tr\xb6 Tri\xd2u Ti\xaan, b\xebi cho r\xbbng ph\xb8o c\xf1a Tri\xd2u Ti\xaan b\xben sang v\xefng bi\xd3n c\xf1a n\xad\xedc n\xb5y";

    let t = Tcvn3::from_bytes(&tcvn3[..]);
    assert_eq!(t.encode_utf8(), unicode);

    let viscii = b"H\xe0n Qu\xafc h\xf4m nay";
    let unicode = "Hàn Quốc hôm nay";
    assert_eq!(Viscii::from_bytes(viscii).encode_utf8(), unicode);
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
