use std::io;
fn main() {
    nomear();
    let mut nome=String::new();
    println!("Digite seu NOME");
    io::stdin().read_line(&mut nome).unwrap();
    println!("Seu nome é {}",nome);
    println!("Digite um numero: ")
    nome.clear();
    io::stdin().read_line(&mut nome).unwrap();
    let mut numero:i32=nome.trim().parse.
    unwrap();
    println!("O numero é {}",nome);
    let mut outroNome=String::new();
    outroNome=nome;
    println("{}",outroNome);
}