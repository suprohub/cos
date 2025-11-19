use core::{cell::SyncUnsafeCell, mem::MaybeUninit};

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

pub static SERIAL: SyncUnsafeCell<MaybeUninit<Serial>> = SyncUnsafeCell::new(MaybeUninit::uninit());

/// Initialize the global serial logger
///
/// # Safety
///
/// Must be called exactly once before any logging macros are used.
/// Must not be called concurrently with any other access to SERIAL.
pub unsafe fn init(serial: Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
    unsafe {
        SERIAL.get().write(MaybeUninit::new(Serial(serial)));
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        // SAFETY: maybe safe? 💀
        let serial = unsafe { &mut (&mut *$crate::log::SERIAL.get()).assume_init_mut().0 };
        ufmt::uwriteln!(serial, $($arg)*).unwrap();
    };
}

#[macro_export]
macro_rules! info_infallible {
    ($($arg:tt)*) => {
        // SAFETY: maybe safe? 💀
        let serial = unsafe { &mut (&mut *$crate::log::SERIAL.get()).assume_init_mut().0 };
        ufmt::uwriteln!(serial, $($arg)*).unwrap();
    };
}

// #[cfg(debug_assertions)] removes debug output in release build
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            // SAFETY: maybe safe? 💀
            let serial = unsafe { &mut (&mut *$crate::log::SERIAL.get()).assume_init_mut().0 };
            ufmt::uwriteln!(serial, $($arg)*).unwrap();
        }
    };
}

#[macro_export]
macro_rules! debug_infallible {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            // SAFETY: maybe safe? 💀
            let serial = unsafe { &mut (&mut *$crate::log::SERIAL.get()).assume_init_mut().0 };
            ufmt::uwriteln!(serial, $($arg)*).unwrap();
        }
    };
}
