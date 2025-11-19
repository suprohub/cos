COS (Calculator OS)
=====

_Think different_

[![dependency status](https://deps.rs/repo/github/suprohub/cos/status.svg)](https://deps.rs/repo/github/suprohub/cos)

**Также доступно на [русском языке](README_RU.md)**

**Rust-powered embedded calculator with joystick interface**

A minimalistic calculator OS for Arduino Nano, using a single vibration motor and a joystick for navigation. Ideal for people with disabilities, educational purposes, or just a unique computing experience.

## Features:
- 🦀 Rust for comfort development and embedded safety
- 🔢 Fixed-point arithmetic engine with rounding (supports numbers up to 2^64)
- 🧮 Basic operations (+, -, ×, ÷) and advanced functions (√, x²)
- 🕹️ Joystick-based navigation through virtual keyboard
- 📳 Vibromotor-based numeric output display via blink patterns

**Why is the joystick selected?**
- Upgrade convenience: changing the behavior of the interface (keyboard layout, navigation logic) is implemented in the firmware — no need to reassemble the electronics, just the firmware.
- Compact: The joystick combines multiple directions and a button in one small component, which reduces size and simplifies installation.

## Hardware Requirements:
- Arduino Nano (New Bootloader)
- Joystick module HW-504 (VRx, VRy, SW)
- Vibromotor
- Maybe serial connection for debugging
- And maybe more in future

## Usage:
Calculator control is intuitive:
- On power-on, the cursor is on the digit 5 of the virtual keyboard
- Moving the joystick changes the cursor position
- Pressing the joystick activates the current key
- After pressing, the cursor returns to the starting position (digit 5)

Virtual keyboard layout:
```
                  Constants
                      |
                  τ γ π e φ √2
                  √ 7 8 9 ÷
Unary operators — - 4 5 6 × — Binary operators
                  x²1 2 3 +
                    . 0 = -
                    C D R
                      |
             Advanced functions
```

Key types:
- Digits - input numbers
- Dot - enables fractional input mode
- Equals - gives the result of a binary operation
- Constants - insert constants
- D (Delete) - remove the last entered character
- C (CE, Clear Entry) - clear the last entered number
- R (Reset) - reset all numbers and the operator
- Advanced functions - varies

Advanced functions may include equation solving using neural networks, photomath, or remembering results of previous calculations, etc.

A more specific layout is defined in the code (configuration will be improved in the future).


## Build Instructions
1. Install prerequisites as described in the [`avr-hal` README] (`avr-gcc`, `avr-libc`, `avrdude`, [`ravedude`]).

2. Run `cargo build` to build the firmware.

3. Run `cargo run` to flash the firmware to a connected board.  If `ravedude`
   fails to detect your board, check its documentation at
   <https://crates.io/crates/ravedude>.

4. `ravedude` will open a console session after flashing where you can interact
   with the UART console of your board.

[`avr-hal` README]: https://github.com/Rahix/avr-hal#readme
[`ravedude`]: https://crates.io/crates/ravedude

## License
Licensed under either of

 - Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 - MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution
Before contribution you need to run `cargo clippy --all-features --fix`, `typos`, `cargo fmt` and `cargo sort`.
For installing `typos` run `cargo binstall typos-cli` (or `cargo install typos-cli`).
For installing `cargo sort` run `cargo binstall cargo-sort` (or `cargo install cargo-sort`).

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
