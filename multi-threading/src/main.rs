mod lookAtThisClass;
use std::thread;
use std::time::Duration;
use rand::random;
use std::fs::File;
use std::io::prelude::*;

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
    let  chad = vec![ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 ];


    let handle = thread::spawn(move || {println!("Cool Ass vector {chad:?}"); chad }) ;
    let mut chad = handle.join().unwrap();
    let handle2 = thread::spawn(move || {for j in 0..chad.len() { chad[j as usize] = random() } chad });
    let chad = handle2.join().unwrap();
    println!("Is it still cool?  {chad:?}");
}
fn mntd(length:u8 ) -> Vec<i128> {
    println!("MTND {length}");

    let mut temp1: Vec<i128> = Vec::new();
    let mut temp: Vec<i128> = Vec::new();
    let mut result: Vec<i128> = Vec::new();


    for i in 0..length  {
        temp1.push(random::<i8>() as i128 );
    }
    for i in 0..length  {
        temp.push(random::<i8>() as i128 );
    }

    for i in 0..length  {
        result.push(temp1[i as usize]  + temp[i as usize] );
    }

    result



}
fn write_vec_to_file(vec:Vec<i128>) -> std::io::Result<()>{
    let mut file = File::create("Vector.txt")?;
    for i in vec {
        let mut temp = i.to_string() + ",";
        file.write(temp.as_bytes())?;
    }
    Ok(())

}



fn main() {
    multi_threaded_fizz_buzz();
    chads1();
    let output = mntd(100);
    write_vec_to_file(output).unwrap();




}