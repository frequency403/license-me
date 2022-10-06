use std::env::args;
use std::io::stdin;
use std::time::Instant;

use crate::output_printer::*;


mod insert;
mod licences;
mod output_printer;
mod search;
mod error_collector;

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

fn arg_modes(arguments: Vec<String>, pmm: &mut PrintMode) -> (bool, bool, bool) {
    let mut license_append_mode: bool = false;
    let mut license_replace_mode: bool = false;
    let mut all_git_dirs_mode: bool = false;
    if arguments.len() > 1 {
        arguments.iter().for_each(|argument| match argument.trim() {
            "-d" => {
                pmm.debug = true;
                pmm.debug_msg("Debug Mode ON")
            }
            "-v" => {
                pmm.verbose = true;
                pmm.verbose_msg("Verbose Mode ON")
            }
            "--append-license" => license_append_mode = true,
            "--replace-license" => license_replace_mode = true,
            "--show-all" => all_git_dirs_mode = true,
            _ => {}
        })
    }
    (license_append_mode, license_replace_mode, all_git_dirs_mode)
}

fn init_search() {
    let sys_time: Instant = Instant::now();
    let mut print_mode: PrintMode = PrintMode::norm();
    let operating_mode: (bool, bool, bool) = arg_modes(args().collect::<Vec<String>>(), &mut print_mode);
    let mut chosen_directories: Vec<&String> = vec![];
    let collection_of_git_dirs: Vec<String> = search::print_git_dirs(operating_mode, &mut print_mode, sys_time);
    if operating_mode.2 {print_mode.normal_msg("\n\nPlease run again for modifying the directories\n");return}
    let input_of_user: String =
        read_input("Enter the number(s) of the repository's to select them: ");
    input_of_user.split_terminator(' ').for_each(|g| {
        if let Ok(int) = g.trim().parse::<isize>() {
            if int.is_positive() {
                if collection_of_git_dirs.len() < int as usize || int == 0 {
                    print_mode.error_msg(format!("Index {} not available", int))
                } else {
                    print_mode.verbose_msg(format!(
                        "Added: {} to processing collection",
                        &collection_of_git_dirs[int as usize - 1]
                    ));
                    chosen_directories.push(&collection_of_git_dirs[int as usize - 1]);
                }
            }
        } else if g == "all" {
            collection_of_git_dirs.iter().for_each(|item| {
                print_mode.verbose_msg(format!("Added: {} to processing collection", item));
                chosen_directories.push(item)
            });
        } else {
            print_mode.error_msg("Invalid input - aborting....");
            std::process::exit(1)
        }
    });
    let p_dirs = insert::insert_license(chosen_directories, operating_mode, &mut print_mode);
    print_mode.err_col.list_errors(p_dirs,&print_mode)
}

fn main() {
    init_search();
}
