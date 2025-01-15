mod bit;
mod coord;
mod file;
mod get;
mod slice;

pub use bit::*;
pub use coord::*;
pub use file::*;
pub use get::*;
pub use slice::*;

pub fn init(is_test: bool) {
    let _ = env_logger::Builder::from_default_env()
        .is_test(is_test)
        .try_init();
}

/// Initialization of program, e.g. logging. For tests, run the test with
/// run_test() instead, which takes care of initialization.
#[macro_export]
macro_rules! init {
    ($a: expr) => {
        $crate::init($a)
    };
    () => {
        $crate::init(false)
    };
}

pub fn teardown() {}

pub fn run_test<T>(test: T) -> ()
where
    T: FnOnce() -> () + std::panic::UnwindSafe,
{
    init!(true);
    let result = std::panic::catch_unwind(|| test());
    assert!(result.is_ok())
}
