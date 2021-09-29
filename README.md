# Embedded Multi-Page HMI

[![crates.io](https://img.shields.io/crates/v/embedded-multi-page-hmi?style=flat-square&logo=rust)](https://crates.io/crates/embedded-multi-page-hmi)
[![docs.rs](https://img.shields.io/badge/docs.rs-embedded--multi--page--hmi-blue?style=flat-square)](https://docs.rs/embedded-multi-page-hmi)
[![license](https://img.shields.io/badge/license-MIT-blue?style=flat-square-blue)](#license)
[![rustc](https://img.shields.io/badge/rustc-1.52+-blue?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![CI status](https://github.com/almedso/embedded-multi-page-hmi/actions/workflows/ci.yml/badge.svg)

An embedded page oriented HMI library supporting a few buttons as input and a constraint display as output on embedded devices.

## Capabilities

- Predefined Input Models using two, three, four or five buttons or a rotary switch.
- Adaptable to different constraints displays: E.g.
  - alpha-numerical
  - ePaper, Oled via [embedded-graphics crate](https://crates.io/crates/embedded-graphics)
- Declarative page structure specification and page transition specification
  - Multiple information pages
  - Continuous page updates and page system triggered page transitions
  - Dedicated startup/ shutdown pages
  - Setting menu, submenu and edit pages

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
embedded-multi-page-hmi = "0.2"
```

Checkout the [example](#example) how to specify page structures and interaction.

## Example

A documented
[demonstration of all capabilities](https://github.com/almedso/embedded-multi-page-hmi/blob/master/examples/simulate-on-host.rs)
is maintained as an example in this crate.
The example application runs on windows, linux and OSX in a terminal using the
[pancurses](https://crates.io/crates/pancurses)
crate.

You can run the example as follow (assuming you have rust installed):

```bash
# Get this repository from github
git clone https://github.com/almedso/embedded-multi-page-hmi.git

cd embedded-multi-page-hmi
# Build and run the example
cargo run --example simulate-on-host
```

## License

This project is licensed under

- MIT License ([`LICENSE.md`](LICENSE.md) or
  [online](https://opensource.org/licenses/MIT))

## Contribution

Feel free and join for contribution.

Checkout [`DESIGN-NOTES.md`](DESIGN-NOTES.md) for design issues and design decisions.

### Future Work

- Edit page for select 1 of many
- Edit page for select multiple of many (list of binary flags)
- Provide examples on raspi with four-button epaper shield
- Move to no-std
  - Replacement for Box with trait objects e.g. RefCell? in page manager
  - Provide example on stmf32 with two-buttons and 16x2 alpha-num display
