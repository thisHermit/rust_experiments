mod datetime;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, ErrorKind};
use std::path::Path;
use csv::{ReaderBuilder, WriterBuilder};
use crate::datetime::datetime_difference;
use std::time::{Duration, Instant};

fn rollup_function(seconds: u128) -> i32 {
    let day = seconds as f32 / 86400.0;
    let y = (day.sqrt() + 0.005 * day.powi(2)).floor() as i32;
    if y == 0 {
        return 1;
    }
    y
}

fn acquire_lock(lock_file: &str, timeout: Duration) -> Result<File, Box<dyn Error>> {
    let start = Instant::now();

    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_file)
        {
            Ok(file) => return Ok(file),
            Err(err) => {
                if err.kind() != ErrorKind::AlreadyExists {
                    return Err(Box::new(err));
                }

                if start.elapsed() > timeout {
                    return Err(format!("Timed out waiting for lock file: {}", lock_file).into());
                }

                println!("Waiting for lock file to be available...");
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "Export.csv";
    let output_file = "Export.csv";
    let lock_file = "file.lock";
    let temp_output = "Export.temp.csv";

    // Try to acquire the lock file with a timeout
    let lock = acquire_lock(lock_file, Duration::from_secs(30))?;

    // Using a scope to ensure the lock is released even if an error occurs
    let result = (|| -> Result<(), Box<dyn Error>> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_path(input_file)?;

        let headers = reader.headers()?.clone();

        // Write to a temporary file first
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_path(temp_output)?;
        writer.write_record(&headers)?;

        let mut previous_date = "2025-03-07 23:39:44".to_string();

        // Get the last record date
        let mut last_date = previous_date.clone();
        for record in reader.records() {
            if let Ok(rec) = record {
                if rec.len() > 2 {
                    last_date = rec[2].to_string();
                }
            }
        }

        // Reset the reader
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
            if record.len() <= 2 {
                continue; // Skip malformed records
            }

            let record_date = record[2].to_string();

            if record_date != previous_date {
                write = false;
                number_blocks += 1;
                let time_difference_from_start = datetime_difference(&record_date, &last_date);
                let rollup_value = rollup_function(time_difference_from_start);
                let time_density = (rollup_value * 299) as u128;
                last_write_time += datetime_difference(&previous_date, &record_date);

                if last_write_time > time_density {
                    write = true;
                    last_write_time = 0;
                } else {
                    deleted_count += 1;
                }

                println!(
                    "Block: {}  | Time Diff: {}s | Rollup Value: {} | Time Density: {} | Write: {} | Acc Time Density: {} | Record Date {} | Deleted: {}",
                    number_blocks,
                    datetime_difference(&previous_date, &record_date),
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

        // Flush and close the writer
        writer.flush()?;
        drop(writer);

        // Rename the temporary file to the output file
        std::fs::rename(temp_output, output_file)?;

        println!("Headers: {:?}", headers);

        Ok(())
    })();

    // Always drop the lock file
    drop(lock);
    std::fs::remove_file(lock_file)?;

    // Return the result from the processing
    result
}