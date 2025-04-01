use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use csv::{ReaderBuilder, WriterBuilder};
use crate::datetime::datetime_difference;

fn rollup_function(seconds: u128) -> i32 {
    let day = seconds as f32 / 86400.0;
    let y = (day.sqrt() + 0.005 * day.powi(2)).floor() as i32;
    if y == 0 {
        return 1;
    }
    y
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "Export.csv";
    let output_file = "filter_export.csv";
    let lock_file = "file.lock";

    // Wait for the lock file to be available
    while Path::new(lock_file).exists() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Create the lock file
    File::create(lock_file)?;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(input_file)?;

    let headers = reader.headers()?.clone();

    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_path(output_file)?;
    writer.write_record(&headers)?;

    let mut previous_date = "2025-03-07 23:39:44".to_string();

    let binding = reader.records().last().unwrap().unwrap();
    let last_date = binding.iter().collect::<Vec<_>>()[2];

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(input_file)?;

    let mut write = true;
    let mut number_blocks = 0;
    let mut last_write_time = 0;
    let mut deleted_count = 0;

    for record in reader.records() {
        if record.is_err() {
            continue;
        }

        let record = record.unwrap();
        let record_date = record.iter().collect::<Vec<_>>()[2].to_string();

        if record_date != previous_date {
            write = false;
            number_blocks += 1;
            let time_difference_from_start = datetime_difference(&record_date, &last_date);
            let rollup_value = rollup_function(time_difference_from_start);
            let time_density = (rollup_value * 299) as u128;
            last_write_time += datetime_difference(&previous_date, &*record_date);

            if last_write_time > time_density {
                write = true;
                last_write_time = 0;
            } else {
                deleted_count += 1;
            }

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

        if write {
            writer.write_record(&record)?;
        }

        previous_date = record_date;
    }

    println!("Headers: {:?}", headers);

    // Remove the lock file
    std::fs::remove_file(lock_file)?;

    Ok(())
}