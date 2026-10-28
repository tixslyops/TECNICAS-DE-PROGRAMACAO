use std::io;

fn main() {
    let mut entrada = String::new();
    println!("Digite um número:");
    io::stdin().read_line(&mut entrada).unwrap();

    let mut numero: i32 = entrada.trim().parse().unwrap();
    for i in 1..=numero {
        println!("{}", i);
    }
}