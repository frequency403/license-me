#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub enum OperatingMode {
    SetNewLicense,
    AppendLicense,
    LicenseReplace,
    ShowAllGitDirs,
    Unlicense, //Deletes the license file and removes links in the readme file.
}
