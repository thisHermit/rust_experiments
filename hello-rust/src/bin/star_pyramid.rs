use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let height = 5;
    for i in 1..height {
        for j in 0..i {
            print!("*");
        }
        println!("");
    }
}
