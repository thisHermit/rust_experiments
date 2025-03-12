use std::thread;
use std::time::Duration;
use rand::random;



// Big Chungus Devs inc
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

fn chads1(){
    let mut chad = vec![ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 ];


    let handle = thread::spawn(move || {println!("Cool Ass vector {chad:?}"); chad }) ;
    let mut chad = handle.join().unwrap();
    let handle2 = thread::spawn(move || {for j in 0..chad.len() { chad[j as usize] = random() } chad });
    let chad = handle2.join().unwrap();
    println!("Is it still cool?  {chad:?}");
}
fn MTND(length:i8 ) -> Vec<i128> {
    println!("MTND {length}");
    return vec![random(), random(), random(), random(), random()];

}



fn main() {
    multi_threaded_fizz_buzz();
    chads1();
    let output = MTND(1);
    println!("now i have fun {{ output: {output:?}  }}") ;





}