#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub enum OperatingMode {
    SetNewLicense,
    AppendLicense,
    LicenseReplace,
    ShowAllGitDirs,
    //TODO UpdateLicense - A Update Routine that determines the current license and replaces it with the newest available from GitHub
    //TODO UpdateAllLicenses - Determines the licenses from all Directories, retrieving new versions for all.
    //TODO Unlicense - Deletes the license file and removes links in the readme file.
}