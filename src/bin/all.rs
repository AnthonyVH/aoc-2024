struct RunResult {
    name: String,
    solution: String,
    duration: std::time::Duration,
}

struct Runner {
    invoker: Box<dyn Fn() -> RunResult>,
}

fn invoke_timed<T>(name: &String, invoker: T) -> RunResult
where
    T: Fn() -> String,
{
    // Invoke a few times and report median.
    const NUM_RUNS: usize = 11;
    let mut durations: Vec<std::time::Duration> = Vec::new();
    let mut solution = String::new();

    for _ in 0..NUM_RUNS {
        let invoke_start = std::time::Instant::now();
        solution = (invoker)();
        durations.push(invoke_start.elapsed());
    }

    durations.sort();
    RunResult {
        name: name.clone(),
        solution,
        duration: durations[durations.len() / 2],
    }
}

macro_rules! create_runner {
    ($day:ident, $part:ident) => {{
        let input_file = format!("{}.txt", stringify!($day));
        let input = util::read_resource(&input_file).unwrap();
        let name = format!(
            "{} - {}",
            heck::AsTitleCase(stringify!($day)),
            heck::AsTitleCase(stringify!($part))
        );

        Runner {
            invoker: Box::new(move || {
                invoke_timed(&name, || format!("{}", aoc_2024::$day::$part(&input)))
            }),
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
        create_runner!(day_14, part_b),
        create_runner!(day_15, part_a),
        create_runner!(day_15, part_b),
        create_runner!(day_16, part_a),
        create_runner!(day_16, part_b),
        create_runner!(day_17, part_a),
        create_runner!(day_17, part_b),
        create_runner!(day_18, part_a),
        create_runner!(day_18, part_b),
        create_runner!(day_19, part_a),
        create_runner!(day_19, part_b),
        create_runner!(day_20, part_a),
        create_runner!(day_20, part_b),
        create_runner!(day_22, part_a),
        create_runner!(day_22, part_b),
        create_runner!(day_23, part_a),
        create_runner!(day_23, part_b),
        create_runner!(day_24, part_a),
        create_runner!(day_24, part_b),
        create_runner!(day_25, part_a),
    ];

    let mut results: Vec<RunResult> = runners.iter().map(|e| (e.invoker)()).collect();
    results.push(RunResult {
        name: String::from("Total"),
        solution: String::default(),
        duration: results.iter().map(|e| e.duration).sum(),
    });

    let max_width_name: usize = results.iter().map(|e| e.name.len()).max().unwrap();
    let max_width_solution: usize = results.iter().map(|e| e.solution.len()).max().unwrap();
    let max_width_time: usize = results
        .iter()
        .map(|e| util::digit_width_base10(e.duration.as_micros() as u64))
        .max()
        .unwrap() as usize;

    for (idx, result) in results.iter().enumerate() {
        if idx == results.len() - 1 {
            println!(
                "{:=^width$}",
                "",
                width = max_width_name + max_width_solution + max_width_time + 10
            );
        }

        let duration_us = result.duration.as_micros();
        println!(
            "{:<width_name$} {:<width_solution$}   {:>width_time$} µs{:>2}",
            format!("{}:", result.name),
            result.solution,
            duration_us,
            if duration_us > 1000 { "!" } else { "" },
            width_name = max_width_name + 1, // Added a ':' to the name.
            width_solution = max_width_solution,
            width_time = max_width_time,
        );
    }
}
