use std::env;
use std::borrow::Borrow;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        println!("No argument supplied.");
        print_help();
        return;
    }
    match args[1].as_ref() {
        "download_project" => download_project(),
        _ => {
            println!("Unknown arguments: {:?}", args);
            print_help();
        },
    };
    println!("{:?}", args);
}

fn download_project(){
    println!("Hello download_project")
}

fn print_help(){

}


