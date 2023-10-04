use crate::alike::is_alike;
use crate::api_communicator::get_all_licenses;
use crate::git_dir::GitDir;
use crate::github_license::GithubLicense;
use crate::operating_mode::OperatingMode;
use crate::output_printer::*;
use crate::settings_file::ProgramSettings;
use crate::walker::init_search;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use std::any::Any;
use std::env::args;
use std::error::Error;
use std::fmt::format;
use std::io::stdin;

// Import the other files
mod alike;
mod api_communicator;
mod error_collector;
mod git_dir;
mod github_license;
mod operating_mode;
mod output_printer;
mod settings_file;
mod walker;

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
        \t\t\tCreates a new one with replacing the complete \"## License\" section in your README.md\n\
        It also gives you the possibility to update your current license.\n\n\
        --show-all\t\tLists all git repository's, regardless of containing a LICENSE file and aborts\n\n\
        --unlicense\t\tDeletes a license from the chosen repositories or chosen repository"
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

fn ask_a_question(question: &str) -> bool {
    matches!(
        read_input(format!("{} [Y/n]:", question).as_str()).as_str(),
        "Y" | "y" | "j" | "J" | "Ja" | "ja" | "Yes" | "yes"
    )
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
            x if x == "help" || x == "-h" || x == "-help" || x == "--help" => print_help(pmm),

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
            "--append-license" => op_mode = OperatingMode::AppendLicense,

            // Replaces the LICENSE file
            "--replace-license" => op_mode = OperatingMode::LicenseReplace,

            // Show all git-repositorys, regardless of the license status
            "--show-all" => op_mode = OperatingMode::ShowAllGitDirs,

            "--unlicense" => op_mode = OperatingMode::Unlicense,
            _ => {}
        })
    }
    op_mode
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Starting time measurement
    let sys_time: tokio::time::Instant = tokio::time::Instant::now();

    // Init the Print mode Struct
    let mut print_mode: PrintMode = PrintMode::norm();

    // Init the SettingsOptions
    let settings: ProgramSettings = ProgramSettings::init(&mut print_mode).await;

    // Check the given arguments
    let operating_mode: OperatingMode = arg_modes(args().collect::<Vec<String>>(), &mut print_mode);

    let progress_bar: ProgressBar = progress_spinner();
    let all_licenses = get_all_licenses(&settings).await?;
    let found_git_dirs: Vec<GitDir> =
        init_search(operating_mode, sys_time, all_licenses.clone()).await;
    progress_bar.finish_and_clear();

    found_git_dirs.iter().enumerate().for_each(|(count, dir)| {
        if operating_mode == OperatingMode::ShowAllGitDirs {
            println!(
                "[License: {}][Readme: {}] {}",
                if dir.license_path.is_some() {
                    ansi_term::Color::Green.paint("true ")
                } else {
                    ansi_term::Color::Red.paint("false")
                },
                if dir.readme_path.is_some() {
                    ansi_term::Color::Green.paint("true ")
                } else {
                    ansi_term::Color::Red.paint("false")
                },
                dir.path
            );
        } else {
            println!("[{}] {}", count + 1, dir.path);
        }
    });

    // If the user just wanted to see how many git directories he has....
    if operating_mode == OperatingMode::ShowAllGitDirs {
        print_mode.normal_msg("\n\nPlease run again for modifying the directories\n");
        // then abort program.
        return Ok(());
    }
    // Using the helper function for reading the Input
    let input_of_user: String =
        read_input("Enter the number(s) of the repository's to select them: ");

    let mut chosen_directories: Vec<GitDir> = vec![];

    // Split string on Whitespace, which creates a vector
    input_of_user.split_terminator(' ').for_each(|g| {
        // For each element of the vector, try to parse the string as int
        if let Ok(int) = g.trim().parse::<isize>() {
            // On purpose used "signed" int's, that the error can be caught here
            if int.is_positive() {
                if found_git_dirs.len() < int as usize || int == 0 {
                    print_mode.error_msg(format!("Index {} not available", int))
                } else {
                    print_mode.verbose_msg(
                        format!(
                            "Added: {} to processing collection",
                            &found_git_dirs[int as usize - 1]
                        ),
                        None,
                    );
                    // Push chosen dirs to the empty collection - also correct the Index with "-1"
                    chosen_directories.push(found_git_dirs[int as usize - 1].clone());
                }
            }
            // Use all directories you found
        } else if g == "all" {
            found_git_dirs.iter().for_each(|item| {
                print_mode.verbose_msg(format!("Added: {} to processing collection", item), None);
                chosen_directories.push(item.clone())
            });
            // If something goes wrong
        } else {
            print_mode.error_msg("Invalid input - aborting....");
            std::process::exit(1)
        }
    });
    // Here starts the main work -> see insert.rs
    //let p_dirs = insert::insert_license(chosen_directories, &operating_mode, &mut print_mode);

    let mut processed_dirs_count: usize = 0;

    for mut choice in chosen_directories.clone() {
        clear_term();
        print_mode.normal_msg(format!(
            "Directory {} of {}",
            processed_dirs_count + 1,
            chosen_directories.len()
        ));
        print_mode.normal_msg(format!(
            "Working on {}\nPath: {}\n\n",
            choice.project_title, choice.path
        ));
        print_mode.normal_msg(format!("Found License: {} | Found Readme: {}", (choice.license_path.is_some() || choice.license.is_some()), choice.readme_path.is_some()));
        if let Some(license) = &choice.license {
            print_mode.normal_msg(format!("Recognized the \"{}\" License!", license.name))
        }
        choice
            .execute_user_action(
                &settings,
                &mut print_mode,
                &operating_mode,
                all_licenses.clone(),
            )
            .await;
        processed_dirs_count += 1;
    }

    // Print all errors the program collected at last.
    print_mode
        .err_col
        .list_errors(processed_dirs_count, &print_mode);
    Ok(())
}

// Using a ProgressBar (spinner) from the crate "ProgressBar"
pub fn progress_spinner() -> ProgressBar {
    // Init main struct
    let p_bar = ProgressBar::new_spinner();
    // Set the style and the tick strings, iterating over all but not the last item every tick
    p_bar.set_style(
        ProgressStyle::with_template("\n{msg}{spinner}")
            .unwrap()
            .tick_strings(&[".   ", " .  ", "  . ", "   .", "  . ", " .  ", " finished!"]),
    );
    // The message shown at {msg}, must be set AFTER declaring the style
    p_bar.set_message("Searching");
    // Using steady tick for eye-friendliness
    p_bar.enable_steady_tick(std::time::Duration::from_secs_f32(0.1));
    // return the bar "object"
    p_bar
}
