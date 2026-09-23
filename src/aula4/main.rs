use std::io;
fn numeros(){
    let mut entrada=String::new();
    println!("Digite um número: ");
    io::stdin().read_line(&mut entrada).unwrap();
    let mut numero:i32=entrada.trim().parse().unwrap();
    println!("O numero é {}",numero);
}
fn somar(a:i32, b:i32){
    let soma:i32 = a+b;
    println!("Resultado da soma:{}", soma);
}
fn subtrair(a:i32, b:i32){
    let sub:i32 = a-b;
    println!("Resultado da subtração:{}",sub);
}
fn multiplicar(a:i32, b:i32){
    let mult:i32 = a*b;
    println!("Resultado da multiplicação:{}",mult);
}
fn dividir(a:i32, b:i32){
    if a>b{
        let divisao:i32 = a/b;
    }else if b>a{
        let divisao:i32 = b/a;  
    }else if a==0 || b==0{
        println!("Divisão por zero");
    }
    // println!("Resultado da divisão:{}",divisao);
}

fn main(){
    while true{
        println!("--MENU--");
        println!("0-Sair");
        println!("1-Somar");
        println!("2-Subtrair");
        println!("3-Multiplicar");
        println!("4-Dividir");
        let mut entrada=String::new();
        entrada.clear();
        println!("Digite uma opção: ");
        io::stdin().read_line(&mut entrada).unwrap();
        let mut opcao:i32=entrada.trim().parse().unwrap();
        if opcao==0{
            println!("Saindo... ");
        }else if opcao==1{
            somar(1,2);
        }else if opcao==2{
            subtrair(2,1);
        }else if opcao==3{
            multiplicar(2,3);
        }else if opcao==4{
            dividir(4,2);    
        }else{
            println!("Entrada inválida");
        }
    }
}
