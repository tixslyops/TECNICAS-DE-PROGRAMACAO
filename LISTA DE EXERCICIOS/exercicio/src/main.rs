use std::io;

fn main() {
    println!("Digite sua idade:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let idade: i32 = input.trim().parse().unwrap();

    if idade >= 18 {
        println!("Maior de idade");
    } else {
        println!("Menor de idade");
    }
}