use std::error::Error;
use std::os::unix::raw::uid_t;
use csv::{ReaderBuilder, WriterBuilder};

// Smart Chungus
fn decompose_date(date: &str) -> Vec<u16>{


    let decomposed_date = date.split(" ").collect::<Vec<&str>>();

    let mut decomposed_date_int: Vec<_> = decomposed_date[0].split("-").map(|x| x.parse::<u16>().unwrap()).collect();
    let mut y_m_d: Vec<_> = decomposed_date[1].split(":").map(|x| x.parse::<u16>().unwrap()).collect();

    decomposed_date_int.append(&mut y_m_d);


    return decomposed_date_int



}
fn seconds_since_start(decomposed_date: Vec<u16>) -> u128 {
    let mut seconds_since_start:u128 = 0;
    seconds_since_start += (decomposed_date[0] -2001) as u128 * 31536000 as u128;
    seconds_since_start += decomposed_date[1] as u128 * 2592000 as u128;
    seconds_since_start += decomposed_date[2] as u128 * 86400 as u128;
    seconds_since_start += decomposed_date[3] as u128 * 3600 as u128;
    seconds_since_start += decomposed_date[4] as u128 * 60 as u128;
    seconds_since_start += decomposed_date[5] as u128 ;

    return seconds_since_start as u128
}

fn datetime_difference(start_date: &str, end_date: &str, measurement: &str  ) -> u128 {

    // dacompose the strings
    let decomposed_start_date = decompose_date(start_date);
    let decomposed_end_date = decompose_date(end_date);

    let seconds_start = seconds_since_start(decomposed_start_date);
    let seconds_end = seconds_since_start(decomposed_end_date);

    if seconds_start > seconds_end { panic!("Start Date bigger then End Date") };

    let seconds_diff = seconds_end -  seconds_start;


    println!("{}", seconds_diff);



    return seconds_diff


}

fn main() -> Result<(), Box<dyn Error>> {
    let  input_file = "Export.csv";
    let  output_file = "filter_export.csv";

    let mut reader = ReaderBuilder::new()
    .has_headers(true)
    .from_path(input_file)?;

    let headers = reader.headers()?.clone();
    datetime_difference("2025-03-07 23:39:44", "2025-03-16 09:26:49", "d");



    println!("Headers: {:?}", headers);
    Ok(())
}
