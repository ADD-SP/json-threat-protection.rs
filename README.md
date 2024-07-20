# JSON-threat-protection.rs

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ADD-SP/json-threat-protection.rs/test.yml?branch=main&style=for-the-badge&label=Fuzzing&link=https%3A%2F%2Fgithub.com%2FADD-SP%2Fjson-threat-protection.rs%2Factions)
![Crates.io Version](https://img.shields.io/crates/v/json-threat-protection?style=for-the-badge&link=https%3A%2F%2Fcrates.io%2Fcrates%2Fjson-threat-protection)
![docs.rs](https://img.shields.io/docsrs/json-threat-protection?style=for-the-badge&link=https%3A%2F%2Fdocs.rs%2Fjson-threat-protection)
![GitHub License](https://img.shields.io/github/license/ADD-SP/json-threat-protection.rs?style=for-the-badge&link=https%3A%2F%2Fgithub.com%2FADD-SP%2Fjson-threat-protection.rs%2Fblob%2Fmain%2FLICENSE)

A Rust library to protect against malicious JSON payloads.

## Features

This crate provides functionality to validate JSON payloads against a set of constraints.

* Maximum depth of the JSON structure.
* Maximum length of strings.
* Maximum number of entries in arrays.
* Maximum number of entries in objects.
* Maximum length of object entry names.
* Whether to allow duplicate object entry names.

## Docs

https://docs.rs/json-threat-protection

## Performance

This crate is designed to be fast and efficient,
and has its own benchmark suite under the `benches` directory.
You can run the benchmarks with the following command:

```bash
JSON_FILE=/path/to/file.json cargo bench --bench memory -- --verbose
```
 
This suite validates the JSON syntax using both this crate and `serde_json`,
you could get your own performance number by specifying the `JSON_FILE` to your dataset.

## Fuzzing

The library is fuzz tested using the `cargo-fuzz` tool.
The fuzzing target is located in the `fuzz` directory.

THe initial set of corpus files are from
[nlohmann/json_test_data](https://github.com/nlohmann/json_test_data).

## Thanks

* [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz): For providing a simple way to fuzz test the library.
* [nlohmann/json_test_data](https://github.com/nlohmann/json_test_data): For providing a initial set of corpus files for fuzzing.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
