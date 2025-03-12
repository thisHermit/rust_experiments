use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut buffer = String::new();
    let mut height = 5;
    stdin.read_line(&mut buffer);
    height = buffer.trim().parse::<usize>().unwrap();

    for i in 1..height+1 {
        buffer = "".to_owned();
        for _j in 0..i {
            buffer = buffer + "*";
        }
        
        print!("{star:<height$}", star=buffer);
        println!("{star:>height$}", star=buffer);
    }
}
