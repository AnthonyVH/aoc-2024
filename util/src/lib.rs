mod bit;
mod coord;
mod file;
mod get;
mod graph;
mod maze;
mod slice;

pub use bit::*;
pub use coord::*;
pub use file::*;
pub use get::*;
pub use graph::*;
pub use maze::*;
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

#[macro_export]
macro_rules! run_day {
    ($day: ident) => {{
        use aoc_2024::$day;

        util::init!();

        // Take last two characters of identifier. Should be the day's number.
        let mut day_repr = stringify!($day);
        day_repr = &day_repr[day_repr.len() - 2..];

        let input_file = format!("{}.txt", stringify!($day));
        let input: String = util::read_resource(&input_file).unwrap();

        println!("[Day {} - Part A] {}", day_repr, aoc_2024::$day::part_a(&input));
        println!("[Day {} - Part B] {}", day_repr, aoc_2024::$day::part_b(&input));
    }};
}
