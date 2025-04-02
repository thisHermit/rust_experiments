fn decompose_date(date: &str) -> Vec<u16>{


    let decomposed_date = date.split(" ").collect::<Vec<&str>>();

    let mut decomposed_date_int: Vec<_> = decomposed_date[0].split("-").map(|x| x.parse::<u16>().unwrap()).collect();
    let mut y_m_d: Vec<_> = decomposed_date[1].split(":").map(|x| x.parse::<u16>().unwrap()).collect();

    decomposed_date_int.append(&mut y_m_d);


    return decomposed_date_int



}
fn seconds_since_start(decomposed_date: Vec<u16>) -> u128 {
    let mut seconds_since_start:u128 = 0;
    seconds_since_start += (decomposed_date[0] -2001) as u128 * 31536000u128;
    seconds_since_start += decomposed_date[1] as u128 * 2592000u128;
    seconds_since_start += decomposed_date[2] as u128 * 86400u128;
    seconds_since_start += decomposed_date[3] as u128 * 3600u128;
    seconds_since_start += decomposed_date[4] as u128 * 60u128;
    seconds_since_start += decomposed_date[5] as u128 ;

    return seconds_since_start as u128
}

pub(crate) fn datetime_difference(start_date: &str, end_date: &str) -> u128 {

    // dacompose the strings
    let decomposed_start_date = decompose_date(start_date);
    let decomposed_end_date = decompose_date(end_date);

    let seconds_start = seconds_since_start(decomposed_start_date);
    let seconds_end = seconds_since_start(decomposed_end_date);

    if seconds_start > seconds_end { return  seconds_start - seconds_end; };

    let seconds_diff = seconds_end -  seconds_start;


    return seconds_diff


}