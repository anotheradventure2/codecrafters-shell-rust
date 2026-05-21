macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        dbg!($($arg)*);
    };
}

pub mod parser;
pub mod path;
pub mod shell;
