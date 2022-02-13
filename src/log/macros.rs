
pub use log::{error as error_0, warn as warn_0, info as info_0, debug as debug_0, trace as trace_0};

#[macro_export] macro_rules! error {
    ($($tts:tt)*) => {
        /*$crate::error_0!($($tts)*);*/
        $crate::log::macros::error_0!("");
    };
}

#[macro_export] macro_rules! warn {
    ($($tts:tt)*) => {
        /*$crate::warn_0!($($tts)*);*/
        $crate::log::macros::warn_0!("");
    };
}

#[macro_export] macro_rules! info {
    ($($tts:tt)*) => {
        /*$crate::info_0!($($tts)*);*/
        $crate::log::macros::info_0!("");
    };
}

#[macro_export] macro_rules! debug {
    ($($tts:tt)*) => {
        /*$crate::debug_0!($($tts)*);*/
        $crate::log::macros::debug_0!("");
    };
}

#[macro_export] macro_rules! trace {
    ($($tts:tt)*) => {
        /*$crate::trace_0!($($tts)*);*/
        $crate::log::macros::trace_0!("");
    };
}
