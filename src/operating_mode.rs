#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub enum OperatingMode {
    SetNewLicense,
    AppendLicense,
    LicenseReplace,
    ShowAllGitDirs,
}