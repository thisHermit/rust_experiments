use std::thread;
use std::time::Duration;
use rand::Rng;
fn multi_threaded_fizz_buzz(){
    let handle = thread::spawn(|| {
        for i in 1..10 {
            if i % 2 == 0 { println!("fizz");}else { println!("buzz"); }
            thread::sleep(Duration::from_millis(1));
        }
    });

    handle.join().unwrap();

    for i in 1..5 {
        println!("CoC now {i}");
        thread::sleep(Duration::from_millis(1));
    }
}



fn main() {
    multi_threaded_fizz_buzz();

    let chad = vec![ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 ];


    let handle = thread::spawn(move || {println!("Cool Ass vector {chad:?}"); chad }) ;
    let chad = handle.join().unwrap();
    println!("Is it still cool?  {chad:?}");



}