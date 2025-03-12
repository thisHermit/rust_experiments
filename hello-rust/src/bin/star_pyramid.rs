use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut buffer = String::new();
    let mut height = 5;
    stdin.read_line(&mut buffer);
    height = buffer.trim().parse::<i32>().unwrap();

    for i in 1..height+1 {
        for _j in 0..i {
            print!("*");
        }
        println!("");
    }
}
