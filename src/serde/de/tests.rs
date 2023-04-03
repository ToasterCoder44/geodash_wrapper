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

const TEST_DATA: &[(&[u8], &[u8])] = &[
    (
        br#"C?xBJImn@fZJJ:\FX|zJBJINyrBnZE[:DI=fANq<dIA:&=Y\<\inl|oTy]fhdoX?i;?jE^c[COe\2zobS}{8;}QE_CMxYLB@E8ZbHd9r;BSZe2A}ll3o>bC>S|JJJJ66"#,
        br#"<?xml version="1.0"?><plist version="1.0" gjver="2.0"><dict><k>key</k><r>1.2</r></dict></plist>"#
    )
];

#[test]
fn decoding_correctly() {
    for (encoded, decoded) in TEST_DATA {
        let mut decoded_by_func = vec![];
        Deserializer::decode(*encoded).unwrap().read_to_end(&mut decoded_by_func).unwrap();
        assert_eq!(decoded, &&decoded_by_func[..]);
    }
}