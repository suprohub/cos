#![no_std]
#![no_main]

use core::panic::PanicInfo;

use arduino_hal::{
    hal::port::PD3,
    port::{Pin, mode::Output},
    prelude::*,
};
use cos::{
    Calculator, Key,
    config::{DEFAULT_POS, FRACTION_COUNT, keyboard_layout},
    debug, info_infallible,
    log::{SERIAL, Serial},
    num::Num,
};
use heapless::Vec;

#[expect(clippy::unwrap_used)]
#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    // SAFETY: This is safe because arduino have only one thread.
    unsafe {
        // TODO: Fix static_mut_refs lint. (idk how fix this)
        #[expect(static_mut_refs)]
        SERIAL.write(Serial(serial));
    }

    let mut vibro = pins.d3.into_output();
    let sw = pins.d2.into_pull_up_input();

    let vrx = pins.a0.into_analog_input(&mut adc);
    let vry = pins.a1.into_analog_input(&mut adc);

    let mut input = InputState::new();
    let mut calc = Calculator::<FRACTION_COUNT>::new();

    loop {
        let pressed = !sw.is_high();
        let dir = read_joystick_direction(vrx.analog_read(&mut adc), vry.analog_read(&mut adc));

        if input.update(dir, pressed) {
            if pressed {
                if let Ok(v) = calc.handle_input(input.key()) {
                    if let Some(v) = v {
                        display_number(&mut vibro, v).unwrap();
                        input.reset_position();
                        continue;
                    }
                } else {
                    blink_err(&mut vibro);
                }
                debug!("pressed {:?}", input.key());
                input.reset_position();
            } else {
                input.update_position(dir);
                debug!("pos: {:?}", input.pos);
            }

            blink(&mut vibro, 1, 250);
        }

        arduino_hal::delay_ms(10);
    }
}

fn display_number(vibro: &mut Pin<Output, PD3>, value: Num<FRACTION_COUNT>) -> Result<(), u8> {
    let mut n = value.0;
    debug!("Value: {}", n);

    arduino_hal::delay_ms(1500);

    if n == 0 {
        blink(vibro, 2, 150);
    } else {
        if n < 0 {
            vibro.set_high();
            arduino_hal::delay_ms(1000);
            vibro.set_low();
            arduino_hal::delay_ms(1500);
        }

        let mut nums = Vec::<_, 19>::new();
        let mut zero_allow = false;
        let mut i = 0u8;

        while n > 0 {
            let digit = (n % 10) as u8;
            debug!("Digit: {}", digit);
            if digit != 0 {
                nums.push(digit)?;
                zero_allow = true;
            } else if zero_allow {
                nums.push(digit)?;
            }

            if i == FRACTION_COUNT - 1 {
                zero_allow = true;
                if !nums.is_empty() {
                    nums.push(10)?;
                }
            }

            n /= 10;
            i += 1;
        }

        nums.reverse();

        for num in nums {
            debug!("Num: {}", num);

            match num {
                0 => blink(vibro, 2, 150),
                10 => blink(vibro, 5, 100),
                n @ 0..=9 => blink(vibro, n, 250),
                _ => {}
            }

            arduino_hal::delay_ms(1500);
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
    Center,
}

struct InputState {
    pos: (u8, u8),
    old_dir: Dir,
    already_pressed: bool,
}

impl InputState {
    fn new() -> Self {
        Self {
            pos: DEFAULT_POS,
            old_dir: Dir::Center,
            already_pressed: false,
        }
    }

    fn update(&mut self, dir: Dir, pressed: bool) -> bool {
        let dir_changed = dir != self.old_dir && dir != Dir::Center;
        self.old_dir = dir;

        let pressed = if !self.already_pressed && pressed {
            self.already_pressed = true;
            true
        } else {
            if self.already_pressed && !pressed {
                self.already_pressed = false;
            }
            false
        };

        dir_changed || pressed
    }

    fn update_position(&mut self, dir: Dir) -> bool {
        match dir {
            Dir::Up => self.pos.1 = self.pos.1.saturating_add(1),
            Dir::Down => self.pos.1 = self.pos.1.saturating_sub(1),
            Dir::Left => self.pos.0 = self.pos.0.saturating_sub(1),
            Dir::Right => self.pos.0 = self.pos.0.saturating_add(1),
            Dir::Center => (),
        }

        false
    }

    fn key(&self) -> Key {
        let mut keyboard_layout = keyboard_layout();
        keyboard_layout.reverse();

        // Get first by y and when by x
        keyboard_layout
            .get(self.pos.1 as usize)
            .and_then(|r| r.get(self.pos.0 as usize).copied())
            .unwrap_or(Key::None)
    }

    fn reset_position(&mut self) {
        self.pos = DEFAULT_POS;
    }
}

fn read_joystick_direction(x: u16, y: u16) -> Dir {
    const MID: u16 = 512;
    const DEADZONE: u16 = 200;

    match (x, y) {
        (x, _) if x > MID + DEADZONE => Dir::Right,
        (x, _) if x < MID - DEADZONE => Dir::Left,
        (_, y) if y > MID + DEADZONE => Dir::Down,
        (_, y) if y < MID - DEADZONE => Dir::Up,
        _ => Dir::Center,
    }
}

fn blink(vibro: &mut Pin<Output, PD3>, count: u8, duration: u16) {
    for _ in 0..count {
        vibro.set_high();
        arduino_hal::delay_ms(duration.into());
        vibro.set_low();
        arduino_hal::delay_ms(duration.into());
    }
}

fn blink_err(vibro: &mut Pin<Output, PD3>) {
    for _ in 0..5 {
        vibro.set_high();
        arduino_hal::delay_ms(50);
        vibro.set_low();
        arduino_hal::delay_ms(50);
    }
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    // Disable interrupts - firmware has panicked so no ISRs should continue running
    avr_device::interrupt::disable();

    // Get the peripherals so we can access the vibro motor.
    //
    // SAFETY: Because main() already has references to the peripherals this is an unsafe
    // operation - but because no other code can run after the panic handler was called,
    // we know it is okay.
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);
    let mut vibro = pins.d3.into_output();

    info_infallible!("Firmware panic!");

    // Accessing the panic info unfortunately means that the optimizer can no longer remove panic
    // messages from the resulting binary.  This leads to an explosion of SRAM usage, quickly
    // surpassing available space.
    //
    // If you need precise panic info, currently your best bet is disabling `overflow-checks` and
    // `debug-assertions` in the build profile and structuring your code such that panics never
    // include a message payload.

    // Example code:
    // uwriteln!(&mut serial, "Panic: {}", info.message().as_str().unwrap_or("Unknown message")).unwrap_infallible();
    // if let Some(loc) = info.location() {
    //     uwriteln!(&mut serial, "Location: {}:{}:{}", loc.file(), loc.line(), loc.column()).unwrap_infallible();
    // }

    loop {
        vibro.toggle();
        arduino_hal::delay_ms(50);
    }
}
