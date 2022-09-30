mod insert;
mod licences;
mod search;
use log::{debug, error, info};
use std::env;
use std::io::stdin;

fn clear_term() {
    print!("\x1B[2J\x1b[1;1H");
}

fn read_input(prompt: &str) -> String {
    let mut s = String::new();
    print!("\n\n");
    println!("{}", prompt);
    if stdin().read_line(&mut s).is_err() {
        std::process::exit(1);
    };
    s.trim().to_string()
}

fn arg_modes(arguments: Vec<String>) -> (bool, bool) {
    let mut license_append_mode: bool = false;
    let mut license_replace_mode: bool = false;
    if arguments.len() > 1 {
        arguments.iter().for_each(|argument| match argument.trim() {
            "-d" => {
                env::set_var("RUST_LOG", "debug");
                debug!("Debug Mode ON");
            }
            "-v" => {
                env::set_var("RUST_LOG", "trace");
                info!("Verbose Mode ON")
            }
            "--append-license" => license_append_mode = true,
            "--replace-license" => license_replace_mode = true,
            _ => {}
        })
    }
    (license_append_mode, license_replace_mode)
}

fn init_search() {
    env::set_var("RUST_LOG", "error");
    env_logger::init();
    let operating_mode: (bool, bool) = arg_modes(env::args().collect::<Vec<String>>());
    let mut colvec: Vec<&String> = vec![];
    let col = search::print_git_dirs(operating_mode);
    let input_of_user: String =
        read_input("Enter the number(s) of the repository's to select them: ");
    input_of_user.split_terminator(' ').for_each(|g| {
        if let Ok(int) = g.trim().parse::<isize>() {
            if int.is_positive() {
                if col.len() < int as usize || int == 0 {
                    error!("Index {} not available", int)
                } else {
                    colvec.push(&col[int as usize - 1]);
                }
            }
        } else if g == "all" {
            col.iter().for_each(|item| colvec.push(item));
        }
    });
    println!(
        "\n\n Done! Processed {} directories successfully!\n",
        insert::insert_license(colvec, operating_mode.1)
    );
}

fn main() {
    init_search();
}
