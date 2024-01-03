use std::fmt::Display;
use std::path::MAIN_SEPARATOR;

use futures::executor::block_on;
use sysinfo::Disks;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use walkdir::WalkDir;

use crate::git_dir::GitDir;
use crate::github_license::GithubLicense;

/// This is an asynchronous function named `init_search` that initializes the
/// searching for git directories in all of the system's disks.
///
/// # Arguments
///
/// * `time` - An instance of `Instant` used to measure the elapsed time taken
///   by the search process.
/// * `licenses` - A vector of GithubLicense. These licenses constraints in
///   finding the git directories.
///
/// # Return
///
/// This function returns a Vector of `GitDir` containing the git directories
/// found in the search process.
///
/// # Behavior
///
/// The function creates a new `System` instance, getting all system's info
/// at once. It then iterates over all the system's disks and for each disk,
/// it spawns a new Tokio task that starts the search operation on that disk.
///
/// These tasks are all stored in a `task_holder` Vector, and the function
/// waits for all these tasks to complete using futures `join_all` function.
/// Once all tasks have finished execution, it iterates over the completed
/// tasks, checks if they're `Ok` and if so, adds the results into the
/// `dirs` Vector of the git directories. If the task's result is an error,
/// it is ignored.
///
/// After all results have been gathered, the function prints out the
/// searching duration in seconds, and eventually returns the `dirs` Vector.
///
/// # Asynchronous nature
///
/// Note that since this is an asynchronous function using async/await
/// syntax, it should be run in an async context.
pub async fn init_search(
    time: Instant,
    licenses: Vec<GithubLicense>,
) -> Vec<GitDir> {
    let disks = Disks::new_with_refreshed_list();
    // let system = System::new_all();
    let mut task_holder: Vec<JoinHandle<Vec<GitDir>>> = vec![];
    disks.iter().for_each(|disk| {
        task_holder.push(tokio::spawn(start_walking(
            disk.mount_point().display().to_string(),
            licenses.clone(),
        )))
    });
    let mut dirs: Vec<GitDir> = vec![];
    futures::future::join_all(task_holder)
        .await
        .iter()
        .filter_map(|dir| {
            if let Ok(result) = dir {
                Some(result)
            } else {
                None
            }
        })
        .for_each(|dir| {
            dir.iter().for_each(|git_dir| {
                dirs.push(git_dir.clone());
            });
        });
    println!("Searching took: {}s", time.elapsed().as_secs());
    dirs
}

/**
 * An asynchronous function that starts walking file directories from the root provided.
 * This operation is limited to a maximum depth of one level into the directory structure.
 * The function concurrently searches the directories and avoids those that contain a '$' symbol or
 * start with '.' after the main separator in their name.
 *
 * # Parameters
 *
 * - `root`: An initial directory (root), represented as any Display-able (T). It's the starting point for the directory walk.
 * - `licences`: A `Vec<GithubLicense>`. It's a vector of GithubLicense items used in the `walk_deeper` function.
 *
 * The function uses `WalkDir` to create an iterator over the entries within a directory which are explored concurrently using 'tokio::spawn'.
 * The future results of these concurrent operations are then consolidated.
 *
 * # Returns
 *
 * - `Vec<GitDir>`: This function returns a Vector of GitDir items extracted during directory walking operation.
 *
 * # Error
 *
 * This function does not explicitly handle errors. However, it suppresses any unhandled errors during the execution of futures.
 *
 * # Example
 *
 * ```rust
 * async fn example() {
 *     let root = "root_directory";    // Your root directory goes here
 *     let licenses: Vec<GithubLicense> = Vec::new(); // Your Github licenses array
 *     let result: Vec<GitDir> = start_walking(root, licenses).await;
 * }
 * ```
 *
 **/
async fn start_walking<T>(
    root: T,
    licences: Vec<GithubLicense>,
) -> Vec<GitDir>
    where
        T: Display,
{
    let mut task_holder: Vec<JoinHandle<Vec<GitDir>>> = vec![];
    WalkDir::new(root.to_string())
        .max_depth(1)
        .into_iter()
        .for_each(|dir| {
            if let Ok(entry) = dir {
                let tmp = entry.path().display().to_string();
                if !tmp.contains('$') || !tmp.split(MAIN_SEPARATOR).collect::<Vec<&str>>()[1].starts_with('.') {
                    task_holder.push(tokio::spawn(walk_deeper(tmp, licences.clone())))
                }
            }
        });
    let mut any_dir: Vec<GitDir> = vec![];
    futures::future::join_all(task_holder)
        .await
        .iter()
        .filter_map(|future| if let Ok(f) = future { Some(f) } else { None })
        .for_each(|res| {
            res.to_vec().iter().for_each(|item| {
                if !any_dir.contains(item) {
                    any_dir.push(item.clone())
                }
            })
        });
    any_dir
}

/// The `walk_deeper` function asynchronously traverses a specified
/// directory root and checks for Git directories without stepping
/// into excluded paths. It performs a recursive search throughout
/// the nested directories.
///
/// # Arguments
///
/// `root: String` - The root directory from where to start
/// the recursive search.
///
/// `licenses: Vec<GithubLicense>` - A vector of `GithubLicense` objects.
///
/// # Returns
///
/// `Vec<GitDir>` - A vector of `GitDir` objects that are initialized
/// from the Git directories found during the root traversal.
///
/// # Behavior
///
/// - Validates each directory path in the root directory.
/// - If it is a Git directory, does not exist inside a .cargo directory,
///   the path without the .git part doesn't contain a . immediately following
///   a path separator, the path does not contain '$' or 'AppData' string,
///   it initializes a `GitDir` object from it and added to the resulting vector.
/// - Uses futures's `block_on` function to initialize the `GitDir` object synchronously.
///
/// # Note
///
/// The function contains commented-out code which accommodates different `OperatingMode`s.
/// Uncommenting and using it might alter the function behavior.
async fn walk_deeper(
    root: String,
    licenses: Vec<GithubLicense>,
) -> Vec<GitDir> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|p_dir| {
            if let Ok(valid_dir) = p_dir {
                let path = valid_dir.path().display().to_string();
                if path.ends_with(format!("{}{}", MAIN_SEPARATOR, ".git").as_str())
                    && !path.contains(".cargo")
                    && !path.replace(".git", "").contains(&format!("{}.", MAIN_SEPARATOR))
                    && !path.contains('$')
                    && !path.contains("AppData")
                {
                    let dir = block_on(GitDir::init(path, Some(licenses.clone())));
                    Some(dir)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}
