extern crate range;

use std::io;
use range::Range;

fn main(){
    println!("Adivinhe o número!");

    let numero_secreto = rand::thread_rng().gen_range(30);

    println!("O número secreto é: {}", numero_secreto);
    println!("Digite o seu palpite:");

    let mut palpite = String::new();

    io::stdin().read_line(&mut palpite)
        .expect("Falha ao ler a linha");

    println!("Você digitou: {}", palpite);
}