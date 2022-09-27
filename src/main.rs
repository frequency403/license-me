mod insert;
mod licences;
mod search;

use std::io::stdin;
use log::error;

fn clear_term() {
    print!("\x1B[2J\x1b[1;1H");
    println!(" ");
}

fn read_input(prompt: &str) -> String {
    let mut s = String::new();
    print!("\n\n");
    println!("{prompt}");
    if stdin().read_line(&mut s).is_err() {
        std::process::exit(1);
    };
    s.trim().to_string()
}



fn main() {
    std::env::set_var("RUST_LOG", "trace"); //set this as "-d" argument
    env_logger::init();
    let mut colvec: Vec<&String> = vec![];
    let col = search::print_git_dirs();
    let input_of_user: String = read_input("Enter the number(s) of the repository's to select them: ");
    input_of_user.split_terminator(' ').for_each(|g| {
        if let Ok(int) = g.parse::<isize>() {
            if int.is_positive() {
                if col.len() < int as usize || int == 0 {
                    error!("Index {int} not available")
                } else {
                    colvec.push(&col[int as usize -1]);
                   // println!("{int} {}", col[int as usize - 1])
                }
            }
        } else if g == "all" {
            col.iter().for_each(|item| colvec.push(item));
        }
    });
    println!("\n\n Done! Processed {} directories successfully!\n", insert::insert_license(colvec));
}
