mod datetime;

use std::error::Error;
use csv::{ReaderBuilder, WriterBuilder};
use crate::datetime::datetime_difference;
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

    let  write = true;
    let mut number_blocks = 0;

    for record in reader.records() {
        if i > 10000 {break}
        if record.is_err() { continue; }

        let record = record.unwrap();
        let record_date = record.iter().collect::<Vec<_>>()[2].to_string(); // Convert to owned String

        if record_date !=  previous_date {
            println!("{:?} vs {}", record_date, previous_date);
            println!("second difference {}", datetime_difference(&previous_date, &*record_date));
            println!("Seconds difference since start {}", datetime_difference(&*record_date, &last_date ));
            println!("Rollup function: {}", rollup_function(datetime_difference(&*record_date, &last_date)));

            number_blocks +=1;
            println!("Number blocks: {}", number_blocks);



            let time_difference_from_start = datetime_difference(&record_date, &last_date);
            let rollup_value = rollup_function(time_difference_from_start);

        }








        writer.write_record(&record).unwrap();

        previous_date = record_date;





        i+=1;
    }



    println!("Headers: {:?}", headers);
    Ok(())
}
