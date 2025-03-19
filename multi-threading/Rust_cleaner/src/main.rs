mod datetime;

use std::error::Error;
use csv::{ReaderBuilder, WriterBuilder};
// Smart
// Chungus

fn rollup_function(seconds:u128) -> i32{
    let day = seconds as f32 / 86400 as f32;

    let y = (day.sqrt() + 0.001 * day.powi(2)).floor() as i32;
    if y == 0 { return 1;}
     y

}


fn main() -> Result<(), Box<dyn Error>> {
    let  input_file = "Export.csv";
    let  output_file = "filter_export.csv";

    let mut reader = ReaderBuilder::new()
    .has_headers(true)
    .from_path(input_file)?;

    let headers = reader.headers()?.clone();

    let mut writer = WriterBuilder::new().has_headers(true).from_path(output_file)?;
    writer.write_record(&headers).expect("Headers no written");



    let mut i = 0;
    let mut previous_date = "2025-03-07 23:39:44".to_string();


    let binding = reader.records().last().unwrap().unwrap();
    let last_date = binding.iter().collect::<Vec<_>>()[2];



    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(input_file)?;



    for record in reader.records() {
        //if i > 10000 {break}
        if record.is_err() { continue; }

        let record = record.unwrap();

        if previous_date != record.iter().collect::<Vec<_>>()[2].to_string() {
            println!("{:?} vs {}", record.iter().collect::<Vec<_>>()[2], previous_date);
            println!("second difference {}", datetime::datetime_difference(&previous_date, record.iter().collect::<Vec<_>>()[2], &record[0]));
            println!("Seconds difference since start {}", datetime::datetime_difference(record.iter().collect::<Vec<_>>()[2],&last_date, &record[1] ));
            println!("Rollup function: {}", rollup_function(datetime::datetime_difference(record.iter().collect::<Vec<_>>()[2],&last_date, &record[1] )));
        }






        writer.write_record(&record).unwrap();
        let record_date = record.iter().collect::<Vec<_>>()[2].to_string(); // Convert to owned String




        previous_date = record_date;



        i+=1;
    }
    
    datetime::datetime_difference("2025-03-07 23:39:44", "2025-03-16 09:26:49", "d");



    println!("Headers: {:?}", headers);
    Ok(())
}
