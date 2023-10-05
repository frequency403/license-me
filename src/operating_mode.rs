use strum_macros::EnumIter;

#[derive(PartialOrd, PartialEq, Copy, Clone, EnumIter, Debug)]
pub enum OperatingMode {
    SetNewLicense,
    AppendLicense,
    LicenseReplace,
    ShowAllGitDirs,
    Unlicense, //Deletes the license file and removes links in the readme file.
}

impl OperatingMode {
    pub fn from_usize(i: usize) -> Option<Self> {
        match i {
            1 => {Some(Self::SetNewLicense)},
            2 => {Some(Self::AppendLicense)},
            3 => {Some(Self::LicenseReplace)},
            4 => {Some(Self::ShowAllGitDirs)},
            5 => {Some(Self::Unlicense)},
            _ => None
        }
    }
}
