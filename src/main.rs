use std::env::args;
use std::future::Future;
use std::io::stdin;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;
use futures::TryFuture;
use sysinfo::{DiskExt, System, SystemExt};
use crate::api_communicator::communicate;
use crate::git_dir::GitDir;
use crate::operating_mode::OperatingMode;

use crate::output_printer::*;
use crate::search::progress_spinner;
use crate::settings_file::ProgramSettings;
use crate::walker::start_walking;

// Import the other files
mod insert;
mod licences;
mod output_printer;
mod search;
mod error_collector;
mod github_license;
mod api_communicator;
mod operating_mode;
mod walker;
mod git_dir;
mod settings_file;

// Prints CLI-Help & exits
// Uses the PrintMode message Method

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

// Clears the Terminal
// Same as "clear"

fn clear_term() {
    print!("\x1B[2J\x1b[1;1H");
}

// Helper Function for reading user-input which prompts the user with a Message
// Also trims whitespaces

fn read_input(prompt: &str) -> String {
    let mut s = String::new();
    println!("\n\n{}", prompt);
    if stdin().read_line(&mut s).is_err() {
        std::process::exit(1);
    };
    s.trim().to_string()
}

// Decides on the given arguments,
// which mode the program is running.
fn arg_modes(arguments: Vec<String>, pmm: &mut PrintMode) -> OperatingMode {
    // Uses a Vec<String> as container for the program Arguments

    let mut op_mode: OperatingMode = OperatingMode::SetNewLicense;
    // If there is an argument.....
    if arguments.len() > 1 {
        // Iterate over every argument, then....
        arguments.iter().for_each(|argument| match argument.trim() {

            // Print help text
            x if x == "help" || x == "-h" || x == "-help" || x == "--help" => { print_help(pmm) }

            // Set the debug mode
            "-d" => {
                pmm.debug = true;
                pmm.debug_msg("Debug Mode ON", None)
            }

            // Set the verbose mode
            "-v" => {
                pmm.verbose = true;
                pmm.verbose_msg("Verbose Mode ON", None)
            }

            // Append/Add a LICENSE to a repo
            "--append-license" => {op_mode = OperatingMode::AppendLicense },

            // Replaces the LICENSE file
            "--replace-license" => {op_mode = OperatingMode::LicenseReplace },

            // Show all git-repositorys, regardless of the license status
            "--show-all" => {op_mode = OperatingMode::ShowAllGitDirs },
            _ => {}
        })
    }
    op_mode
}

#[tokio::main]
async fn main() {
    // if let Some(license) = communicate().await {
    //     println!("{}",license.set_username_and_year().body)
    // } else {
    //     println!("Error")
    // }
    // let vecs = tokio::spawn(start_walking("C:\\"));
    // let c = vecs.await.unwrap();
    let mut print_mode: PrintMode = PrintMode::norm();
    let settings: ProgramSettings = ProgramSettings::init(&mut print_mode).await;
    println!("{settings}");
    return;

    // END OF TEST SECTION
    // STARTING "NORMAL" SECTION HERE


    // Starting time measurement
    let sys_time: Instant = Instant::now();

    // Init the Print mode Struct
    let mut print_mode: PrintMode = PrintMode::norm();

    // Check the given arguments
    let operating_mode: OperatingMode = arg_modes(args().collect::<Vec<String>>(), &mut print_mode);

    // Init empty Vector for the directories, that the user wants to edit
    let mut chosen_directories: Vec<&String> = vec![];

    // Collection of found directories, containing the absolute path as string
    let collection_of_git_dirs: Vec<String> = search::print_git_dirs(&operating_mode, &mut print_mode, sys_time).await;

    // If the user just wanted to see how many git directories he has....
    if operating_mode == OperatingMode::ShowAllGitDirs {
        print_mode.normal_msg("\n\nPlease run again for modifying the directories\n");
        // then abort program.
        return;
    }

    // Using the helper function for reading the Input
    let input_of_user: String = read_input("Enter the number(s) of the repository's to select them: ");

    // Split string on Whitespace, which creates a vector
    input_of_user.split_terminator(' ').for_each(|g| {
        // For each element of the vector, try to parse the string as int
        if let Ok(int) = g.trim().parse::<isize>() {
            // On purpose used "signed" int's, that the error can be caught here
            if int.is_positive() {
                if collection_of_git_dirs.len() < int as usize || int == 0 {
                    print_mode.error_msg(format!("Index {} not available", int))
                } else {
                    print_mode.verbose_msg(format!(
                        "Added: {} to processing collection",
                        &collection_of_git_dirs[int as usize - 1]
                    ), None);
                    // Push chosen dirs to the empty collection - also correct the Index with "-1"
                    chosen_directories.push(&collection_of_git_dirs[int as usize - 1]);
                }
            }
            // Use all directories you found
        } else if g == "all" {
            collection_of_git_dirs.iter().for_each(|item| {
                print_mode.verbose_msg(format!("Added: {} to processing collection", item), None);
                chosen_directories.push(item)
            });
            // If something goes wrong
        } else {
            print_mode.error_msg("Invalid input - aborting....");
            std::process::exit(1)
        }
    });
    // Here starts the main work -> see insert.rs
    let p_dirs = insert::insert_license(chosen_directories, &operating_mode, &mut print_mode);

    // Print all errors the program collected at last.
    print_mode.err_col.list_errors(p_dirs.await, &print_mode)
}
