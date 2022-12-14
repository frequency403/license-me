use std::env::args;
use std::io::stdin;
use std::time::Instant;

use crate::output_printer::*;


mod insert;
mod licences;
mod output_printer;
mod search;
mod error_collector;

fn print_help(pmm: &PrintMode) {
    pmm.normal_msg(
        "LICENSE-ME\t\tA CLI-TOOL FOR LICENSING YOUR GIT REPOSITORYS!\n\n\
        USAGE: ./license-me[.EXE] [OPTIONS]\n\n\
        help, -h, -help, --help\t\t\tShows this prompt\n\n\
        -d\t\t\t\t\tturns on \"DEBUG\" mode\n\n\
        -v\t\t\t\t\tturns on \"VERBOSE\" mode\n\n\
        If you Invoke the Program like this, you will get extra output and you can see what it does.\n\
        In this mode, with or without debug/verbose mode, the program will find all repos WITHOUT a \"LICENSE\" file in it.\n
        It will let you Create a \"LICENSE\" file, and it will create a README.md if none is found.\n
        If a README.md is found, it will only append the link to your license to the end of your README.md\n\n\n\
        [MODE-CHANGING OPTIONS]\n\n\n\
        These options will list all git repository's with a \"LICENSE\" file in it\n\n\n\
        --append-license\tAdds a license to the chosen directory, and appends a Link to the end of README.md\n\n\
        --replace-license\tIt will delete ALL license-like files in your chosen directory.\n\
        \t\t\tCreates a new one with replacing the complete \"## License\" section in your README.md\n\n\
        --show-all\t\tLists all git repository's, regardless of containing a LICENSE file and aborts\n"
    );
    std::process::exit(0);
}

fn clear_term() {
    print!("\x1B[2J\x1b[1;1H");
}

fn read_input(prompt: &str) -> String {
    let mut s = String::new();
    println!("\n\n{}", prompt);
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
            x if x == "help" || x == "-h" || x == "-help" || x == "--help" => { print_help(pmm) }
            "-d" => {
                pmm.debug = true;
                pmm.debug_msg("Debug Mode ON", None)
            }
            "-v" => {
                pmm.verbose = true;
                pmm.verbose_msg("Verbose Mode ON", None)
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
    if operating_mode.2 {
        print_mode.normal_msg("\n\nPlease run again for modifying the directories\n");
        return;
    }
    let input_of_user: String = read_input("Enter the number(s) of the repository's to select them: ");
    input_of_user.split_terminator(' ').for_each(|g| {
        if let Ok(int) = g.trim().parse::<isize>() {
            if int.is_positive() {
                if collection_of_git_dirs.len() < int as usize || int == 0 {
                    print_mode.error_msg(format!("Index {} not available", int))
                } else {
                    print_mode.verbose_msg(format!(
                        "Added: {} to processing collection",
                        &collection_of_git_dirs[int as usize - 1]
                    ), None);
                    chosen_directories.push(&collection_of_git_dirs[int as usize - 1]);
                }
            }
        } else if g == "all" {
            collection_of_git_dirs.iter().for_each(|item| {
                print_mode.verbose_msg(format!("Added: {} to processing collection", item), None);
                chosen_directories.push(item)
            });
        } else {
            print_mode.error_msg("Invalid input - aborting....");
            std::process::exit(1)
        }
    });
    let p_dirs = insert::insert_license(chosen_directories, operating_mode, &mut print_mode);
    print_mode.err_col.list_errors(p_dirs, &print_mode)
}

fn main() {
    init_search();
}
