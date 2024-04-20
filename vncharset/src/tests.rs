use super::{Viscii, Tcvn3};

#[test]
fn main() {
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
