use super::*;

use std::io::Read;

// use xorstream::Transformer as XorReader;
// use base64::{write::EncoderWriter as Base64Writer, Engine};
// use base64::engine::general_purpose::URL_SAFE;
// use base64::engine::GeneralPurpose;
// use libflate::gzip::Encoder as GzipWriter;
// use quick_xml::{
//     Writer as XmlWriter,
//     Result as XmlResult
// };

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
    valid_events: Vec<Event>,
    is_valid: bool
}

fn test_data<'a>() -> Vec<TestSample> {
    vec![
        TestSample {
            encoded: br#"C?xBJImn@fZJJ:\FX|zJBJINyrBnZE[:DI=fANq<dIA:&=Y\<\inl|oTy]fhdoX?i;?jE^c[COe\2zobS}{8;}QE_CMxYLB@E8ZbHd9r;BSZe2A}ll3o>bC>S|JJJJ66"#.to_vec(),
            decoded: br#"<?xml version="1.0"?><plist version="1.0" gjver="2.0"><dict><k>key</k><r>1.2</r></dict></plist>"#.to_vec(),
            valid_events: vec![
                Event::XmlVersion(String::from("1.0")),
                Event::PlistStart { plist_version: String::from("1.0"), gj_version: String::from("2.0") },
                Event::DictStart,
                Event::Key(String::from("key")),
                Event::Real(String::from("1.2")),
                Event::DictEnd,
                Event::Eof
            ],
            is_valid: true
        }
    ]
}

#[test]
fn decodes_correctly() {
    for sample in test_data() {
        let mut decoded_by_func = vec![];
        Deserializer::decode(&sample.encoded[..]).unwrap().read_to_end(&mut decoded_by_func).unwrap();
        assert_eq!(sample.decoded, &decoded_by_func[..]);
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
