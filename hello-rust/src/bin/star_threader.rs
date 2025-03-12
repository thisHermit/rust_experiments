use std::io::{self, BufRead};
use std::thread;
use std::time::Duration;

fn main() {
    let stdin = io::stdin();
    let mut buffer = String::new();
    let _ = stdin.read_line(&mut buffer);
    let height = buffer.trim().parse::<usize>().unwrap();

    let handle = thread::spawn(|| {
        for i in 1..10 {
            if i % 2 == 0 { println!("fizz");}else { println!("buzz"); }
            thread::sleep(Duration::from_millis(1));
        }
    });

    handle.join().unwrap();
    pyramider(height);
    
}

fn pyramider(height : usize) { 
    let mut buffer = String::new();
    for i in 1..height+1 {
        buffer = "".to_owned();
        for _j in 0..i {
            buffer = buffer + "*";
        }
        
        print!("{star:<height$}", star=buffer);
        println!("{star:>height$}", star=buffer);
    }
}
