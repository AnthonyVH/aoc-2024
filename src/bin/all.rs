use comfy_table::Table;

struct RunResult {
    name: String,
    solution: String,
    duration: std::time::Duration,
}

struct Runner {
    invoker: Box<dyn Fn() -> RunResult>,
}

fn invoke_timed<T>(name: String, invoker: T) -> RunResult
where
    T: Fn() -> String,
{
    // Invoke a few times and report median.
    const NUM_RUNS: usize = 11;
    let mut duration: std::time::Duration = std::time::Duration::MAX;
    let mut solution = String::new();

    for _ in 0..NUM_RUNS {
        let invoke_start = std::time::Instant::now();
        solution = (invoker)();
        duration = std::cmp::min(duration, invoke_start.elapsed());
    }

    // Use the minimum seen duration.
    RunResult {
        name: name,
        solution,
        duration: duration
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
                invoke_timed(name.clone(), || {
                    format!("{}", aoc_2024::$day::$part(&input))
                })
            }),
        }
    }};
}

/// Calculate the mean and standard deviation.
fn mean_and_std<T, U>(data: T) -> Option<(f64, f64)>
where
    T: Iterator<Item = U>,
    U: Into<f64>,
{
    // NOTE: This method is numerically unstable. Doesn't matter here though.
    let mut sum = 0_f64;
    let mut var = 0_f64;
    let mut count = 0_usize;

    for value in data {
        count += 1;

        let float: f64 = value.into();
        sum += float as f64;
        var += float as f64 * float as f64;
    }

    match count {
        0 => None,
        _ => {
            let mean = sum / count as f64;
            let std = ((var / count as f64) - mean * mean).sqrt();

            Some((mean, std))
        }
    }
}

fn print_table(results: &[RunResult]) {
    // Grab some statistics for highlighting later on.
    let (mean, std) = mean_and_std(results.iter().map(|e| e.duration.as_micros() as u32)).unwrap();

    // Bunch of helper lambdas.
    let make_cell = |value: String, idx: usize| {
        let cell = comfy_table::Cell::new(value);

        match idx % 2 {
            1 => cell.bg(comfy_table::Color::Rgb {
                r: 60,
                g: 60,
                b: 60,
            }),
            _ => cell,
        }
    };

    let make_cells = |value: &RunResult, idx: usize| {
        return [
            make_cell(value.name.clone(), idx),
            make_cell(value.solution.clone(), idx),
            make_cell(value.duration.as_micros().to_string(), idx),
        ];
    };

    let highlighted_cells = |value: &RunResult, idx: usize| {
        let mut cells = make_cells(value, idx);

        // Highlight runtime if it's an outlier.
        let duration = value.duration.as_micros() as f64;
        let outlier = (duration - mean).abs() / std;

        // TODO: These thresholds are very random...
        if outlier > 0.75_f64 {
            let mut new_cell = cells[2].clone();
            new_cell = new_cell.fg(comfy_table::Color::Red);

            if outlier > 1.5_f64 {
                new_cell = new_cell.add_attribute(comfy_table::Attribute::Bold);
            }

            cells[2] = new_cell;
        }

        cells
    };

    // Basic table setup.
    // The bottom corners are modified, to allow printing a row for the totals
    // underneath, separated from the rest of the table with a full line.
    let mut table = Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL_CONDENSED)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .set_style(comfy_table::TableComponent::BottomLeftCorner, '├')
        .set_style(comfy_table::TableComponent::BottomRightCorner, '┤');
    table.set_header(["Problem", "Solution", "Time [µs]"]);

    // Add results for all runners.
    table
        .column_mut(2)
        .unwrap()
        .set_cell_alignment(comfy_table::CellAlignment::Right);

    for (idx, result) in results.iter().enumerate() {
        table.add_row(highlighted_cells(result, idx));
    }

    println!("{}", table);

    // Add row with totals.
    let mut footer_table = Table::new();
    footer_table
        .load_preset(comfy_table::presets::UTF8_FULL_CONDENSED)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .remove_style(comfy_table::TableComponent::TopBorder)
        .remove_style(comfy_table::TableComponent::TopBorderIntersections)
        .remove_style(comfy_table::TableComponent::TopLeftCorner)
        .remove_style(comfy_table::TableComponent::TopRightCorner)
        .remove_style(comfy_table::TableComponent::VerticalLines)
        .set_style(comfy_table::TableComponent::BottomBorderIntersections, '─');

    // Didn't set headers, so must first add rows before columns exist and can
    // be modified.
    let total = RunResult {
        name: String::from("Total"),
        solution: String::default(),
        duration: results.iter().map(|e| e.duration).sum(),
    };
    footer_table.add_row(make_cells(&total, table.row_count()));

    // Ensure column widths match those for the results.
    for (idx, width) in table.column_max_content_widths().iter().enumerate() {
        let padding_width = table.column(idx).unwrap().padding_width();
        let total_width = comfy_table::Width::Fixed(*width + padding_width);

        footer_table
            .column_mut(idx)
            .unwrap()
            .set_constraint(comfy_table::ColumnConstraint::Absolute(total_width));
    }

    footer_table
        .column_mut(2)
        .unwrap()
        .set_cell_alignment(comfy_table::CellAlignment::Right);

    println!("{}", footer_table);
}

fn main() {
    util::init!();

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
        create_runner!(day_21, part_a),
        create_runner!(day_21, part_b),
        create_runner!(day_22, part_a),
        create_runner!(day_22, part_b),
        create_runner!(day_23, part_a),
        create_runner!(day_23, part_b),
        create_runner!(day_24, part_a),
        create_runner!(day_24, part_b),
        create_runner!(day_25, part_a),
    ];

    let results: Vec<_> = runners.iter().map(|e| (e.invoker)()).collect();
    print_table(&results);
}
