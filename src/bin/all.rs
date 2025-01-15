struct RunResult {
    solution: usize,
    duration: std::time::Duration,
}

struct Runner {
    name: String,
    invoker: Box<dyn Fn() -> RunResult>,
}

fn invoke_timed<T>(invoker: T) -> RunResult
where
    T: Fn() -> usize,
{
    // Invoke a few times and report median.
    const NUM_RUNS: usize = 11;
    let mut durations: Vec<std::time::Duration> = Vec::new();
    let mut solution = 0;

    for _ in 0..NUM_RUNS {
        let invoke_start = std::time::Instant::now();
        solution = (invoker)();
        durations.push(invoke_start.elapsed());
    }

    durations.sort();
    RunResult {
        solution,
        duration: durations[durations.len() / 2],
    }
}

macro_rules! create_runner {
    ($day:ident, $part:ident) => {{
        let input_file = format!("{}.txt", stringify!($day));
        let input = util::read_resource(&input_file).unwrap();
        let day_str: String = stringify!($day).to_string();

        Runner {
            name: format!(
                "Day {} - Part {}",
                day_str[day_str.len() - 2..].to_string(),
                stringify!($part).to_uppercase().pop().unwrap()
            ),
            invoker: Box::new(move || invoke_timed(|| aoc_2024::$day::$part(&input) as usize)),
        }
    }};
}

fn main() {
    let runners: Vec<Runner> = vec![
        create_runner!(day_01, part_a),
        create_runner!(day_01, part_b),
        create_runner!(day_02, part_a),
        create_runner!(day_02, part_b),
        create_runner!(day_03, part_a),
        create_runner!(day_03, part_b),
        create_runner!(day_04, part_a),
        create_runner!(day_04, part_b),
        create_runner!(day_05, part_a),
        create_runner!(day_05, part_b),
        create_runner!(day_06, part_a),
        create_runner!(day_06, part_b),
        create_runner!(day_07, part_a),
        create_runner!(day_07, part_b),
        create_runner!(day_08, part_a),
        create_runner!(day_08, part_b),
        create_runner!(day_09, part_a),
        create_runner!(day_09, part_b),
        create_runner!(day_10, part_a),
        create_runner!(day_10, part_b),
        create_runner!(day_11, part_a),
        create_runner!(day_11, part_b),
        create_runner!(day_12, part_a),
        create_runner!(day_12, part_b),
        create_runner!(day_13, part_a),
        create_runner!(day_13, part_b),
        create_runner!(day_14, part_a),
        //create_runner!(day_14, part_b),
        create_runner!(day_15, part_a),
        create_runner!(day_15, part_b),
        create_runner!(day_16, part_a),
        create_runner!(day_16, part_b),
    ];

    for runner in runners {
        let result = (runner.invoker)();
        let us = result.duration.as_micros();
        println!(
            "{}: {:<15}{:10} µs{:>2}",
            runner.name,
            result.solution,
            us,
            if us > 1000 { "!" } else { "" }
        );
    }
}
