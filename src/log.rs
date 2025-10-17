use core::mem::MaybeUninit;

use arduino_hal::{
    Usart,
    hal::port::{PD0, PD1},
    pac::USART0,
    port::{
        Pin,
        mode::{Input, Output},
    },
};

pub struct Serial(pub Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>);

// SAFETY: This impl is safe because arduino have only one thread.
unsafe impl Send for Serial {}
// SAFETY: ^
unsafe impl Sync for Serial {}

pub static mut SERIAL: MaybeUninit<Serial> = MaybeUninit::uninit();

// TODO: Fix static_mut_refs lint. (idk how fix this)

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        #[allow(static_mut_refs)]
        // SAFETY: maybe safe? 💀
        let serial = unsafe { &mut (*$crate::log::SERIAL.as_mut_ptr()).0 };

        ufmt::uwriteln!(serial, $($arg)*).unwrap();
    };
}

#[macro_export]
macro_rules! info_infallible {
    ($($arg:tt)*) => {
        #[allow(static_mut_refs)]
        // SAFETY: maybe safe? 💀
        let serial = unsafe { &mut (*$crate::log::SERIAL.as_mut_ptr()).0 };
        ufmt::uwriteln!(serial, $($arg)*).unwrap_infallible();
    };
}

// #[cfg(debug_assertions)] removes debug output in release build
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            #[expect(static_mut_refs)]
            // SAFETY: maybe safe? 💀
            let serial = unsafe { &mut (*$crate::log::SERIAL.as_mut_ptr()).0 };
            ufmt::uwriteln!(serial, $($arg)*).unwrap();
        }
    };
}

#[macro_export]
macro_rules! debug_infallible {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            #[allow(static_mut_refs)]
            // SAFETY: maybe safe? 💀
            let serial = unsafe { &mut (*$crate::log::SERIAL.as_mut_ptr()).0 };
            ufmt::uwriteln!(serial, $($arg)*).unwrap_infallible();
        }
    };
}
