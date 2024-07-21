# JSON-threat-protection.rs

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ADD-SP/json-threat-protection.rs/test.yml?branch=main&style=for-the-badge&label=Fuzzing)](https://github.com/ADD-SP/json-threat-protection.rs/actions)
[![Crates.io Version](https://img.shields.io/crates/v/json-threat-protection?style=for-the-badge)](https://crates.io/crates/json-threat-protection)
[![docs.rs](https://img.shields.io/docsrs/json-threat-protection?style=for-the-badge&link=https%3A%2F%2Fdocs.rs%2Fjson-threat-protection)](https://docs.rs/json-threat-protection)
[![GitHub License](https://img.shields.io/github/license/ADD-SP/json-threat-protection.rs?style=for-the-badge)](LICENSE)

A Rust library to protect against malicious JSON payloads.

*This project is not a parser, and never give you the deserialized JSON Value!*

## Features

This crate provides functionality to validate JSON payloads against a set of constraints.

* Maximum depth of the JSON structure.
* Maximum length of strings.
* Maximum number of entries in arrays.
* Maximum number of entries in objects.
* Maximum length of object entry names.
* Whether to allow duplicate object entry names.

The typical use case for this crate is to validate JSON payloads
before the bussiness logic of your application that is deployed
in a separated place.

## Docs

https://docs.rs/json-threat-protection

## Performance

This crate is designed to be fast and efficient,
and has its own benchmark suite under the `benches` directory.
You can run the benchmarks with the following command:

```bash
JSON_FILE=/path/to/file.json cargo bench --bench memory -- --verbose
```
 
This suite validates the JSON syntax and checks the above constraints.

For comparison, the `serde_json` crate is used to parse the JSON
and get the `serde_json::Value` and travel through the JSON structure
to check the above constraints.

Here are the table of the results of the benchmark suite for different datasets:

[*Data Source*](data/)

|             Dataset           | Size | serde_json + Validation | json-threat-protection | Faster     | Comment |
|-------------------------------|------|-------------------------|------------------------|------------|---------|
| kernel_stargazers.json        | 1.2M | 12.996 ms               | 8.8530 ms              | 1.46x      | 1000 stargazers JSON information from [torvalds/linux](https://github.com/torvalds/linux) |
| kernel_stargazers_small.json  | 568K | 5.8825 ms               | 3.7504 ms              | 1.56x      | 472 stargazers JSON information from [torvalds/linux](https://github.com/torvalds/linux) |
| kernel_commits.json           | 4.6M | 45.059 ms               | 29.682 ms              | 1.51x      | 1000 commits JSON infomation from [torvalds/linux](https://github.com/torvalds/linux) |
| tokio_issues.json             | 5.1M | 61.935 ms               | 33.959 ms              | 1.82x      | 1000 issues JSON information from [tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| tokio_forks.json              | 6.1M | 90.984 ms               | 45.686 ms              | 1.99x      | 1000 forks JSON information from [tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| tokio_workflow_runs.json      | 15M  | 221.89 ms               | 103.65 ms              | 2.14x      | 1000 workflow runs JSON information from [tokio-rs/tokio](https://github.com/tokio-rs/tokio) |

It is expected that the `json-threat-protection` crate
will be faster than the `serde_json` crate
because it never store the deserialized JSON Value in memory,
which reduce the cost on memory allocation and deallocation.

As you can see from the table,
the `json-threat-protection` crate is faster than the `serde_json + Validation` crate
for all datasets, but the number depends on the dataset.
So you could get your own performance number by
specifying the `JSON_FILE` to your dataset.

*Note: Comparing with `serde_json + Validation` is not fair*
*as the `serde_json` crate is a parser but the `json-threat-protection` crate is not.*
*The comparison is just for reference.*
*And it is better to compare with other similar crates,*
*but I haven't found that yet, and this is also the reason for developing this crate.*

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
