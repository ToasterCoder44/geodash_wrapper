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
    is_valid: bool
}

fn test_data<'a>() -> Vec<TestSample> {
    vec![
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
                DeEvent::DictEnd,
                DeEvent::Eof
            ],
            is_valid: true
        },
        TestSample {
            encoded: b"C?xBJL[ZGfZJJ;9@XZyJBIJN}rGqlLr8|LY3_IGHIOo^sDn<?FM_os\\MFfxg;~xO\\8[I}f|lHQ8bNLo<l[a&r~9mZ[a|CZeSF_9eJzOALAQGJJJJ".to_vec(),
            decoded: br#"<?xml version="1.0"?><plist version="1.2" gjver="1.9"><dict></dict></plist>"#.to_vec(),
            valid_events: vec![
                DeEvent::XmlVersion(String::from("1.0")),
                DeEvent::PlistStart { plist_version: String::from("1.2"), gj_version: String::from("1.9") },
                DeEvent::DictStart,
                DeEvent::DictEnd,
                DeEvent::Eof
            ],
            is_valid: true
        },
        TestSample {
            encoded: b"C?xBJNSZGfZJJ;\\@\\|zJBIJJy<GxJoZ&c\\;2_N]x\\Bf@nC|m[T<EOND~}c<@MQ[38?>L\\SYF|^}@z9<@BEs[23N\\f^?>Fflf[^2~OIb|_NRJJJJ6".to_vec(),
            decoded: br#"<?xml version="0.9"?><plist version="1.0" gjver="1.9"><dict /></plist>"#.to_vec(),
            valid_events: vec![
                DeEvent::XmlVersion(String::from("0.9")),
                DeEvent::PlistStart { plist_version: String::from("1.0"), gj_version: String::from("1.9") },
                DeEvent::DictStart,
                DeEvent::DictEnd,
                DeEvent::Eof
            ],
            is_valid: true
        }
    ]
}

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
        if !sample.is_valid {
            let next = deserializer.next();
            assert!(next.is_err(), "Expected error, got `{:?}`", next.unwrap());
        }
    }
}
