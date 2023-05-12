use super::*;

use std::io::Read;

// use std::io::Write;
// use base64::Engine;
// use base64::engine::general_purpose::URL_SAFE;
// use libflate::gzip::Encoder as GzipWriter;

// fn encode(input: &[u8]) -> Vec<u8> {
//     let mut writer = GzipWriter::new(vec![]).unwrap();
//     writer.write_all(input).unwrap();
//     let encoded = writer.finish().unwrap();
//     let encoded = URL_SAFE.encode(&encoded.0);
//     encoded.as_bytes().into_iter().map(|b| b ^ 11).collect()
// }

struct TestSample {
    encoded: Vec<u8>,
    decoded: Vec<u8>,
    valid_events: Vec<DeEvent>,
    assert_err: Option<Box<dyn FnOnce(DeError)>>
}

fn test_data() -> Vec<TestSample> { vec![
    TestSample {
        encoded: b"C?xBJB<ZGfZJJ:SE^Z=HFIJN;@x;[ZH]N}:jcl\x7f?HjFE\\\\`G9j8L?:}@M8&<i>BQfe?{ff3Z>_\\[\x7f~3~olA\x7f`i\\h:hq}&dT\\:s};?fhIGQO|bDZ\\`@O}TNIDo~]hLa@~myc]>]9:HDn>IRy<@bNQ8}X_rHeBCR9~GnF[f@J<[{RJJJJ6".to_vec(),
        decoded: br#"<?xml version="1.0"?><plist version="1.0" gjver="2.0"><dict><k>real</k><r>1.23</r><k>int</k><i>52363</i><k>string</k><s>Lorem ipsum</s></dict></plist>"#.to_vec(),
        valid_events: vec![
            DeEvent::XmlVersion(String::from("1.0")),
            DeEvent::PlistStart { plist_version: String::from("1.0"), gj_version: String::from("2.0") },
            DeEvent::DictStart,
            DeEvent::Key(String::from("real")),
            DeEvent::Real(String::from("1.23")),
            DeEvent::Key(String::from("int")),
            DeEvent::Integer(String::from("52363")),
            DeEvent::Key(String::from("string")),
            DeEvent::String(String::from("Lorem ipsum")),
            DeEvent::DictEnd
        ],
        assert_err: None
    },
    TestSample {
        encoded: b"C?xBJIxrF\\ZJJ9\\EX|zJFJsNy:ArljOyF]>H8Bx\\boR[ERaC:<QGE3[|CxDlmiilic3}[mjL&gBzjl]e;F\x7f&zbR8Gs2xr[re^>Il;\x7fNNz;ZTI[JzbEG>sr{|_Mc8r:Zqo_]RI]sLeCT`I\\D:c\x7fzAJJJJ".to_vec(),
        decoded: br#"<?xml version="Version1"?><plist version="Version2" gjver="test test"><dict><k>real</k><r>Text1</r><k>int</k><i>Text 2</i></dict></plist>"#.to_vec(),
        valid_events: vec![
            DeEvent::XmlVersion(String::from("Version1")),
            DeEvent::PlistStart { plist_version: String::from("Version2"), gj_version: String::from("test test") },
            DeEvent::DictStart,
            DeEvent::Key(String::from("real")),
            DeEvent::Real(String::from("Text1")),
            DeEvent::Key(String::from("int")),
            DeEvent::Integer(String::from("Text 2")),
            DeEvent::DictEnd
        ],
        assert_err: None
    },
    TestSample {
        encoded: b"C?xBJL[ZGfZJJ;9@XZyJBIJN}rGqlLr8|LY3_IGHIOo^sDn<?FM_os\\MFfxg;~xO\\8[I}f|lHQ8bNLo<l[a&r~9mZ[a|CZeSF_9eJzOALAQGJJJJ".to_vec(),
        decoded: br#"<?xml version="1.0"?><plist version="1.2" gjver="1.9"><dict></dict></plist>"#.to_vec(),
        valid_events: vec![
            DeEvent::XmlVersion(String::from("1.0")),
            DeEvent::PlistStart { plist_version: String::from("1.2"), gj_version: String::from("1.9") },
            DeEvent::DictStart,
            DeEvent::DictEnd
        ],
        assert_err: None
    },
    TestSample {
        encoded: b"C?xBJNSZGfZJJ;\\@\\|zJBIJJy<GxJoZ&c\\;2_N]x\\Bf@nC|m[T<EOND~}c<@MQ[38?>L\\SYF|^}@z9<@BEs[23N\\f^?>Fflf[^2~OIb|_NRJJJJ6".to_vec(),
        decoded: br#"<?xml version="0.9"?><plist version="1.0" gjver="1.9"><dict /></plist>"#.to_vec(),
        valid_events: vec![
            DeEvent::XmlVersion(String::from("0.9")),
            DeEvent::PlistStart { plist_version: String::from("1.0"), gj_version: String::from("1.9") },
            DeEvent::DictStart,
            DeEvent::DictEnd
        ],
        assert_err: None
    },
    TestSample {
        encoded: b"C?xBJL=xD9ZJJ:\\F\\|zJBIYN\x7frB~dA\x7f&S3n22HIFr;bAg{3gmmlsFCFRO\x7f\x7f<H&@jq&_biz_znfeIY8J{\x7f:Zxj2gL=\x7fGI`sxq9LDBFXlfO3=HmzJyJEDinz][bZn&89Qji|JJJJ66".to_vec(),
        decoded: br#"<?xml version="1.0"?><plist version="1.0" gjver="2.0"><dict><k>bool1</k><t /><k>bool2</k><t></t></dict></plist>"#.to_vec(),
        valid_events: vec![
            DeEvent::XmlVersion(String::from("1.0")),
            DeEvent::PlistStart { plist_version: String::from("1.0"), gj_version: String::from("2.0") },
            DeEvent::DictStart,
            DeEvent::Key(String::from("bool1")),
            DeEvent::True,
            DeEvent::Key(String::from("bool2")),
            DeEvent::True,
            DeEvent::DictEnd
        ],
        assert_err: None
    },
    TestSample {
        encoded: b"C?xBJJ;E[LZJJs8@X|dJBJlJ;j~NI&b8]f3_R^YMY~o[9x8J|2]Ma<\x7fgz3sI`C|N]>x2ZiQfOMT|J?@A\x7fy3{JJJJ".to_vec(),
        decoded: br#"<plist version="1.0" gjver="2.0"></plist>"#.to_vec(),
        valid_events: vec![],
        assert_err: Some(Box::new(|err| {
            println!("{err}");
        }))
    }
] }

// #[test]
// fn _temp() {
//     let x = encode(br#"something, xml"#);
//     println!("###::{:?}::###", from_utf8(&x).unwrap());
// }

#[test]
fn decodes_correctly() {
    for sample in test_data() {
        let mut decoded_by_func = vec![];
        let mut reader = Deserializer::decode(&sample.encoded[..]).unwrap();
        reader.read_to_end(&mut decoded_by_func).unwrap();
        assert_eq!(sample.decoded, decoded_by_func);
    }
}

#[test]
fn emits_events_correctly() {
    for sample in test_data() {
        let mut deserializer = Deserializer::from_reader(&sample.encoded[..]).unwrap();
        for event in sample.valid_events {
            assert_eq!(Arc::new(event), deserializer.next().unwrap());
        }

        let next = deserializer.next();
        if let Some(assert_err) = sample.assert_err {
            assert_err(next.unwrap_err());
        } else {
            assert_eq!(*next.unwrap(), DeEvent::Eof);
        }
    }
}
