use std::env::args;
use std::error::Error;
use std::io::stdin;
use std::ops::Range;
use std::process;
use futures::executor::block_on;
use indicatif::{ProgressBar, ProgressStyle};
use strum::IntoEnumIterator;
use crate::api_communicator::get_all_licenses;
use crate::git_dir::GitDir;
use crate::github_license::GithubLicense;
use crate::operating_mode::OperatingMode;
use crate::output_printer::*;
use crate::settings_file::ProgramSettings;
use crate::walker::init_search;

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


/// Creates a progress spinner and returns it.
///
/// # Examples
///
/// ```
/// let spinner = progress_spinner();
/// ```
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



/// Prints the help message for the CLI tool.
///
/// # Arguments
///
/// * `pmm` - A reference to the `PrintMode` struct.
///
/// # Example
///
/// ```
/// use crate::PrintMode;
///
/// let pmm = PrintMode::new();
/// print_help(&pmm);
/// ```
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
        [CONFIGURATION OPTIONS]\n\n\n\
        --initial-configuration\t\tWill ask you two questions, with one required for the program to run (username)\n\n\
        --github-user\t\tSets the github-user in the settings file\n\n\
        --github-token\t\tSets the token for deactivating the API Limit\n\n\n\
        [MODE-CHANGING OPTIONS]\n\n\n\
        These options will list all git repository's with a \"LICENSE\" file in it\n\n\n\
        --append-license\tAdds a license to the chosen directory, and appends a Link to the end of README.md\n\n\
        --replace-license\tIt will delete ALL license-like files in your chosen directory.\n\
        \t\t\tCreates a new one with replacing the complete \"## License\" section in your README.md\n\
        \t\t\tIt also gives you the possibility to update your current license.\n\n\
        --show-all\t\tLists all git repository's, regardless of containing a LICENSE file and aborts\n\n\
        --unlicense\t\tDeletes a license from the chosen repositories or chosen repository"
    );
    process::exit(0);
}


/// Clears the terminal screen.
///
/// This function clears the terminal screen by printing escape sequences to the console. It uses the ANSI escape sequences
/// `\x1B[2J` and `\x1b[1;1H` to clear the screen and move the cursor to the top left corner respectively.
///
/// # Examples
///
/// ```
/// use my_library::clear_term;
///
/// fn main() {
///     // Clear the terminal screen
///     clear_term();
/// }
/// ```
fn clear_term() {
    print!("\x1B[2J\x1b[1;1H");
}



/// Reads user input from the command line.
///
/// # Arguments
///
/// * `prompt` - A message to display as a prompt to the user.
///
/// # Returns
///
/// A `String` containing the user's input, with leading and trailing whitespace removed.
/// If reading the input fails, the function will exit the process with an exit code of 1.
///
/// # Examples
///
/// ```rust
/// let input = read_input("Please enter your name: ");
/// println!("Hello, {}!", input);
/// ```
fn read_input(prompt: &str) -> String {
    let mut s = String::new();
    println!("\n\n{}", prompt);
    if stdin().read_line(&mut s).is_err() {
        process::exit(1);
    };
    s.trim().to_string()
}

/// # ask_a_question
///
/// A function that asks a question and expects a yes or no answer from the user.
///
/// ## Arguments
/// * `question` - A string slice that represents the question to be asked to the user.
///
/// ## Returns
/// * `bool` - A boolean value indicating whether the user's answer was yes (`true`) or no (`false`).
///
/// ## Examples
/// ```
/// use std::io;
///
/// fn read_input(prompt: &str) -> String {
///     let mut input = String::new();
///     io::stdin().read_line(&mut input).unwrap();
///     input.trim().to_string()
/// }
///
/// let question = "Do you like pizza?";
/// let answer = ask_a_question(question);
/// if answer {
///     println!("Great, you like pizza!");
/// } else {
///     println!("Oh no, you don't like pizza!");
/// }
/// ```
fn ask_a_question(question: &str) -> bool {
    matches!(
        read_input(format!("{} [Y/n]:", question).as_str()).as_str(),
        "Y" | "y" | "j" | "J" | "Ja" | "ja" | "Yes" | "yes"
    )
}


/// Determines the operating mode based on the input arguments.
///
/// # Arguments
///
/// * `arguments` - A vector of strings representing the program arguments.
/// * `pmm` - A mutable reference to the `PrintMode` struct.
/// * `settings_file` - A mutable reference to the `ProgramSettings` struct.
///
/// # Returns
///
/// The determined operating mode as a value of the `OperatingMode` enum.
fn arg_modes(arguments: Vec<String>, pmm: &mut PrintMode, settings_file: &mut ProgramSettings) -> OperatingMode {
    // Uses a Vec<String> as container for the program Arguments

    let mut op_mode: OperatingMode = OperatingMode::SetNewLicense;
    // If there is an argument.....
    if arguments.len() > 1 {
        // Iterate over every argument, then....
        arguments.iter().enumerate().for_each(|(count, argument)| match argument.trim() {
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

            "--initial-configuration" => {
                settings_file.github_user = read_input("Enter your Github-Username (Otherwise the program will not work!):");
                settings_file.github_api_token = if ask_a_question("Do you have a token?:") {
                    Some(read_input("Enter the token:"))
                } else {
                    None
                };
                if settings_file.github_api_token.is_none() {
                    pmm.normal_msg("Get one here: https://github.com/settings/tokens\nRemember that you have an request limit for the Github API!");
                } else {
                    pmm.normal_msg("Restart the Program to get started with license-me!")
                }
                if let Err(s) = block_on(settings_file.save_changes()) {
                    pmm.error_msg(s)
                } else {
                    pmm.normal_msg("Settings successfully updated");
                }
                process::exit(0)
            }

            "--github-token" => {
                settings_file.github_api_token = Some(arguments[count].clone());
                if let Err(s) = block_on(settings_file.save_changes()) {
                    pmm.error_msg(s)
                } else {
                    pmm.normal_msg("Settings successfully updated");
                }
                process::exit(0)
            }

            "--github-user" => {
                settings_file.github_user = arguments[count].clone();
                if let Err(s) = block_on(settings_file.save_changes()) {
                    pmm.error_msg(s)
                } else {
                    pmm.normal_msg("Settings successfully updated");
                }
                process::exit(0)
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


/// Extracts and validates a number as a string.
///
/// This function takes a `num_as_str` parameter which is a string representation of a number
/// and a `len_of_vec` parameter which represents the length of a vector. The function attempts
/// to parse the `num_as_str` as an `isize`. If the parsing is successful, it checks if the
/// parsed number is positive and within the range of `len_of_vec`. If the number is not positive
/// or out of range, it returns an `Err` with the error message "NumNotPositiveOrInRange". If the
/// parsing fails, it returns an `Err` with the error message "NumParsingError". If everything is
/// successful, it returns an `Ok` with the parsed number as `usize`.
///
/// # Arguments
///
/// * `num_as_str` - A string representation of a number.
/// * `len_of_vec` - Length of a vector.
///
/// # Returns
///
/// An `Result` which contains either the parsed number as `usize` or an `Err` with an error message.
///
/// # Examples
///
/// ```rust
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let num_as_str = "5";
///     let len_of_vec = 10;
///     let result = extract_and_validate_num(num_as_str, len_of_vec)?;
///     assert_eq!(result, 5);
///     Ok(())
/// }
/// ```
///
/// ```rust
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let num_as_str = "15";
///     let len_of_vec = 10;
///     let result = extract_and_validate_num(num_as_str, len_of_vec)?;
///     // Returns an Err("NumNotPositiveOrInRange")
///     Ok(())
/// }
/// ```
fn extract_and_validate_num(num_as_str: &str, len_of_vec: usize) -> Result<usize, Box<dyn Error>> {
    if let Ok(int) = num_as_str.trim().parse::<isize>() {
        if !int.is_positive() || int > len_of_vec as isize {
            return Err(Box::from("NumNotPositiveOrInRange"));
        }
        Ok(int as usize)
    } else {
        Err(Box::from("NumParsingError"))
    }
}

/// Presents the directories and their information based on the operating mode and print mode.
///
/// # Arguments
///
/// * `directories` - A vector of `GitDir` structs representing the directories.
/// * `operating_mode` - An `OperatingMode` enum representing the mode of operation.
/// * `print_mode` - A `PrintMode` enum representing the mode of printing.
///
/// # Returns
///
/// * A `Result` containing a vector of `usize` representing the selected directories, or a boxed dynamic `Error` trait object.
fn present_dirs(directories: &Vec<GitDir>, operating_mode: &OperatingMode, print_mode: &PrintMode) -> Result<Vec<usize>, Box<dyn Error>> {
    directories.iter().enumerate().for_each(|(count, dir)| {
        match operating_mode {
            OperatingMode::SetNewLicense => {
                if dir.license_path.is_none() || dir.license.is_none() {
                    println!("[{}] {}", count + 1, dir.path);
                }
            }
            OperatingMode::ShowAllGitDirs => {
                println!(
                    "[License: {}][Readme: {}] {}",
                    PrintMode::colored_bools(&(dir.license_path.is_some() || dir.license.is_some())),
                    PrintMode::colored_bools(&dir.readme_path.is_some()),
                    dir.path
                );
            }
            _ => {
                if dir.license_path.is_some() || dir.license.is_some() {
                    println!("[{}] {}", count + 1, dir.path);
                }
            }
        }
    });

    // If the user just wanted to see how many git directories he has....
    if operating_mode == &OperatingMode::ShowAllGitDirs {
        print_mode.normal_msg("\n\nPlease run again for modifying the directories\n");
        process::exit(0);
    }

    let mut input_of_user: Vec<usize> = vec![];

    match read_input("Enter the number(s) of the repository's to select them: ").as_str() {
        x if x.contains(", ") => {
            x.split(", ").for_each(|e| {
                if let Ok(parsed) = extract_and_validate_num(e, directories.len()) {
                    input_of_user.push(parsed)
                }
            })
        }
        x if x.contains(' ') => {
            x.split(' ').for_each(|e| {
                if let Ok(parsed) = extract_and_validate_num(e, directories.len()) {
                    input_of_user.push(parsed)
                }
            })
        }
        x if x.contains(',') => {
            x.split(',').for_each(|e| {
                if let Ok(parsed) = extract_and_validate_num(e, directories.len()) {
                    input_of_user.push(parsed)
                }
            })
        }
        x if x.contains('-') => {
            let mut range: Range<usize> = 69420..69421; // In Honor of Omer
            x.split('-').for_each(|e| {
                if let Ok(parsed) = extract_and_validate_num(e, directories.len()) {
                    if range.start == 69420 {
                        range.start = parsed
                    } else {
                        range.end = parsed + 1
                    }
                }
            });
            range.for_each(|choice| input_of_user.push(choice))
        }
        x if x.contains("all") => { directories.iter().enumerate().for_each(|entry| input_of_user.push(entry.0)) }
        x if x.parse::<usize>().is_ok() => { input_of_user.push(extract_and_validate_num(x, directories.len())?) }
        _ => {}
    }
    Ok(input_of_user)
}

/// Main function that performs recursive processing on the given Git directories.
///
/// # Arguments
///
/// * `found_git_dirs` - A mutable vector of `GitDir` representing the found Git directories
/// * `all_licenses` - A vector of `GithubLicense` representing all available licenses
/// * `print_mode` - The `PrintMode` to use for printing messages
/// * `settings` - The `ProgramSettings` to use for configuring the program
/// * `operating_mode` - The `OperatingMode` to use for determining the mode of operation
///
/// # Returns
///
/// A `Result` containing the number of processed directories on success, or a `Box<dyn Error>` on failure.
///
/// # Examples
///
/// ```
/// // Initialize variables
/// let mut found_git_dirs: Vec<GitDir> = Vec::new();
/// let all_licenses: Vec<GithubLicense> = vec![
///     GithubLicense { name: "License 1" },
///     GithubLicense { name: "License 2" },
/// ];
/// let print_mode = PrintMode::Normal;
/// let settings = ProgramSettings { /* settings configuration */ };
/// let operating_mode = OperatingMode::Unlicense;
///
/// // Call the recursive_main function
/// let result = recursive_main(&mut found_git_dirs, all_licenses, print_mode, settings, operating_mode);
///
/// // Handle the result
/// match result {
///     Ok(processed_dirs_count) => {
///         println!("Total processed directories: {}", processed_dirs_count);
///     },
///     Err(error) => {
///         eprintln!("An error occurred: {}", error);
///     },
/// }
/// ```
async fn recursive_main(found_git_dirs: &mut Vec<GitDir>, all_licenses: Vec<GithubLicense>, mut print_mode: PrintMode, settings: ProgramSettings, operating_mode: OperatingMode) -> Result<usize, Box<dyn Error>> {
    let mut processed_dirs_count: usize = 0;
    let chosen_dirs = present_dirs(found_git_dirs, &operating_mode, &print_mode)?;

    for chosen_nums in &chosen_dirs {
        let chosen_dir = &mut found_git_dirs[chosen_nums - 1];
        clear_term();
        if operating_mode == OperatingMode::Unlicense {
            print_mode.normal_msg(format!("Deleting license from {} ...", chosen_dir.project_title))
        } else {
            print_mode.normal_msg(format!(
                "Directory {} of {}",
                processed_dirs_count + 1,
                &chosen_dirs.len()
            ));
            print_mode.normal_msg(format!(
                "Working on {}\nPath: {}",
                ansi_term::Color::Blue.paint(&chosen_dir.project_title), &chosen_dir.path
            ));
            print_mode.normal_msg(format!(
                "Found License: {} | Found Readme: {}",
                PrintMode::colored_bools(&(chosen_dir.license_path.is_some() || chosen_dir.license.is_some())),
                PrintMode::colored_bools(&chosen_dir.readme_path.is_some())
            ));
            if let Some(license) = &chosen_dir.license {
                print_mode.normal_msg(format!("Recognized the \"{}\" License!", license.name));
            } else {
                print_mode.normal_msg("\n\n");
            }
        }
        chosen_dir
            .execute_user_action(
                &settings,
                &mut print_mode,
                &operating_mode,
                all_licenses.clone(),
            )
            .await;
        processed_dirs_count += 1;
    }
    Ok(processed_dirs_count)
}

fn print_initial() {
    clear_term();
    println!("\t\t|--------------------------------------------|");
    println!("\t\t|-----------------License Me-----------------|");
    println!("\t\t|--------------------------------------------|");
    println!("\t\t|------------Support us On GitHub------------|");
    println!("\t\t|-https://github.com/frequency403/license-me-|");
    println!("\t\t|---Idea and Programming by Oliver Schantz---|");
    println!("\t\t|--------------------------------------------|\n\n");
}

/// Runs the main program
///
/// This function is the entry point of the program. It performs the following steps:
/// 1. Calls the `print_initial` function to print the initial message.
/// 2. Starts measuring the system time.
/// 3. Initializes the `PrintMode` struct.
/// 4. Initializes the `ProgramSettings` struct by calling the `init` method.
/// 5. Initializes variables to keep track of the processed directories count, licenses, and found git directories.
/// 6. Enters a loop that continues until there are no remaining licenses or git directories.
/// 7. If both the licenses and git directories are empty, it calls the `get_all_licenses` function to populate the `all_licenses` vector and the `init
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print_initial();

    // Starting time measurement
    let sys_time: tokio::time::Instant = tokio::time::Instant::now();

    // Init the Print mode Struct
    let mut print_mode: PrintMode = PrintMode::norm();

    // Init the SettingsOptions
    let mut settings: ProgramSettings = ProgramSettings::init(&mut print_mode).await;

    let mut processed_dirs_count = 0;

    // Check the given arguments
    let mut operating_mode: OperatingMode = arg_modes(args().collect::<Vec<String>>(), &mut print_mode, &mut settings);

    let mut all_licenses: Vec<GithubLicense> = vec![];
    let mut found_git_dirs: Vec<GitDir> = vec![];

    loop {
        if all_licenses.is_empty() && found_git_dirs.is_empty() {
            let progress_bar: ProgressBar = progress_spinner();
            all_licenses = get_all_licenses(&settings).await?;
            found_git_dirs = init_search(sys_time, all_licenses.clone()).await;
            progress_bar.finish_and_clear();
        }

        if let Ok(num) = recursive_main(&mut found_git_dirs, all_licenses.clone(), print_mode.clone(), settings.clone(), operating_mode).await {
            processed_dirs_count += num;
            if ask_a_question("Do you want to repeat the Process?") {
                OperatingMode::iter().enumerate().for_each(|(c, n)| {
                    print_mode.normal_msg(format!("[{}] {:#?}", c + 1, n));
                });
                if let Ok(num) = read_input("Choose your operating mode:").parse::<usize>() {
                    if let Some(enumeration) = OperatingMode::from_usize(num) {
                        operating_mode = enumeration;
                        print_mode.normal_msg(format!("Chosen mode: {:#?}", operating_mode));
                        continue;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    // Print all errors the program collected at last.
    print_mode
        .err_col
        .list_errors(processed_dirs_count, &print_mode);
    Ok(())
}

