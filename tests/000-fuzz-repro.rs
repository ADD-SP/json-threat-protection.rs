// mod utils;

// use core::panic;

// use json_threat_protection as jtp;
// use serde_json::Value;

// #[allow(dead_code)]
// fn print_str(data: &[u8]) {
//     let str = String::from_utf8_lossy(data);
//     eprintln!("{:?}", str);
// }

// #[allow(dead_code)]
// fn validate_by_serde_json(data: &[u8]) {
//     let _v = serde_json::from_slice::<Value>(data).unwrap();
// }

// #[test]
// fn fuzz() {
//     // let data = std::fs::read("fuzz/artifacts/simple/crash-a9ab76df22b06440fbe285569b4a6e199f6b1311").unwrap();
//     let data = [10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 32, 102, 97, 108, 115, 101];

//     // let binding = String::from_utf8_lossy(&data);
//     // let parts = binding.split_at(77);
//     // eprintln!("part[0]: {:#}", parts.0);
//     // eprintln!("part[1]: {:#}", parts.1);

//     println!("{:?}", data);
//     print_str(&data);
//     // validate_by_serde_json(&data);

//     let validator = jtp::from_slice(&data);
//     validator.validate().unwrap();

//     let buf_reader = std::io::BufReader::new(&data[..]);
//     let validator = jtp::from_reader(buf_reader);
//     // validator.validate().unwrap();

//     panic!("done");

//     // let data = br#"true"#;
//     // let stream = ByteStream::new(data);
//     // let validator = Validator::new(stream);
//     // validator.validate().unwrap();

//     // let data = br#"false"#;
//     // let stream = ByteStream::new(data);
//     // let validator = Validator::new(stream);
//     // validator.validate().unwrap();

//     // let data = br#"null"#;
//     // let stream = ByteStream::new(data);
//     // let validator = Validator::new(stream);
//     // validator.validate().unwrap();

//     // let data = br#""fdfd""#;
//     // let stream = ByteStream::new(data);
//     // let validator = Validator::new(stream).with_max_string_length(4);
//     // validator.validate().unwrap();
// }
