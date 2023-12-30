# fcm
[![Cargo tests](https://github.com/rj76/fcm-rust/actions/workflows/test.yml/badge.svg)](https://github.com/panicbit/fcm-rust/actions/workflows/test.yml)
[![Coveralls](https://img.shields.io/coveralls/panicbit/fcm-rust.svg?style=flat-square)][coveralls]
[![Crates.io Version](https://img.shields.io/crates/v/fcm.svg?style=flat-square)][crates.io]
[![Crates.io Downloads](https://img.shields.io/crates/dv/fcm.svg?style=flat-square)][crates.io]
[![Crates.io License](https://img.shields.io/crates/l/fcm.svg?style=flat-square)][crates.io]

[travis]: https://travis-ci.org/panicbit/fcm-rust
[coveralls]: https://coveralls.io/github/panicbit/fcm-rust

## v1 API

This fork is a rewrite to use Google's HTTP v1 API.

# Credentials

This library expects the Google credentials JSON location to be 
defined as `GOOGLE_APPLICATION_CREDENTIALS` in the `.env` file.

## Examples

Check out the examples directory for a simple sender.
