fn main() {
    println!("Hello, world!");
    for value in 1..11{
        print!("{}: ", value);
        if value % 2 == 0 {
            print!("Fizz")
        }
        if value % 5 == 0 {
            print!("Buzz")
        }
        println!()
    }
}