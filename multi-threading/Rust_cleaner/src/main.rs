mod datetime;

use crate::datetime::datetime_difference;
use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

// Smart
// Chungus

/// Computes a rollup value based on time in seconds.
/// The formula scales with time to adjust granularity.
fn rollup_function(seconds: u128) -> i32 {
    let day = seconds as f32 / 86400.0; // Convert seconds to days

    let y = (day.sqrt() + 0.005 * day.powi(2)).floor() as i32; // Custom rollup formula
    if y == 0 {
        return 1;
    }
    y
}
fn create_locking(lock: &str) -> std::io::Result<()> {
    let file = File::create(lock).unwrap();
    Ok(())
}

fn locking_function_start() -> bool {
    // If the locking file doesn't exist it returns False
    // if it does it return true

    let filename = "file.lock";

    if !std::path::Path::new(filename).exists() {
        let _ = create_locking(filename).is_ok();
        return false;
    }
    true
}
fn locking_function_end() -> std::io::Result<()> {
    std::fs::remove_file("file.lock")?;
    Ok(())
}

fn csv_rollup_parser(input_file: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    // Open CSV reader and writer
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(input_file)?;

    let headers = reader.headers()?.clone();

    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_path(output_file)?;
    writer.write_record(&headers).expect("Headers not written");

    let mut i = 0;
    let mut previous_date = "2025-03-07 23:39:44".to_string(); // Initial reference date

    // Retrieve the last date in the CSV for reference in time calculations
    let binding = reader.records().last().unwrap().unwrap();
    let last_date = binding.iter().collect::<Vec<_>>()[2];

    // Reopen the reader to start processing from the beginning
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(input_file)?;

    let mut write = true;
    let mut number_blocks = 0;
    let mut last_write_time = 0;
    let mut deleted_count = 0;

    for record in reader.records() {
        if record.is_err() {
            continue; // Skip records with errors
        }

        let record = record.unwrap();
        let record_date = record.iter().collect::<Vec<_>>()[2].to_string(); // Extract the timestamp

        if record_date != previous_date {
            // Time has changed, indicating a new time block
            write = false;

            number_blocks += 1; // Count number of distinct time blocks
            let time_difference_from_start = datetime_difference(&record_date, &last_date);
            let rollup_value = rollup_function(time_difference_from_start);

            let time_density = (rollup_value * 299) as u128;
            last_write_time += datetime_difference(&previous_date, &*record_date);

            // If accumulated time surpasses the threshold, allow writing
            if last_write_time > time_density {
                write = true;
                last_write_time = 0;
            } else {
                deleted_count += 1
            }

            // Print the key values whenever the time changes
            println!(
                "Block: {}  | Time Diff: {}s | Rollup Value: {} | Time Density: {} | Write: {} | Acc Time Density: {} | Record Date {} | Deleted: {}",
                number_blocks,
                datetime_difference(&previous_date, &*record_date),
                rollup_value,
                time_density,
                write,
                last_write_time,
                record_date,
                deleted_count
            );
        }

        // Write record if the write condition is met
        if write {
            writer.write_record(&record).unwrap();
        }

        previous_date = record_date; // Update previous_date for next comparison
    }

    println!("Headers: {:?}", headers); // Final debug statement
    Ok(())
}

fn main() {
    let input_file = "Export.csv";
    let output_file = "temp_Export.csv";
    let mut done = false;
    let mut iterations = 0;

    while done != true {
        if iterations != 0 {
            sleep(Duration::new(5, 0));
        }
        if locking_function_start() != true {
            csv_rollup_parser(input_file, output_file).expect("Failed");
            println!("triggered");
            done = true;
        }
        iterations += 1;
    }
    std::fs::rename("temp_Export.csv", input_file).expect("Cannot rename");

    locking_function_end().expect("Cannot Remove Locking");
}
