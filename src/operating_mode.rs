use strum_macros::EnumIter;

/// Enum representing different operating modes for a software license.
///
/// This enum is used to determine the behavior of software license operations.
/// It implements several traits, such as `PartialOrd`, `PartialEq`, `Copy`, `Clone`, `EnumIter`, and `Debug`.
#[derive(PartialOrd, PartialEq, Copy, Clone, EnumIter, Debug)]
pub enum OperatingMode {
    SetNewLicense,
    AppendLicense,
    LicenseReplace,
    ShowAllGitDirs,
    Unlicense, //Deletes the license file and removes links in the readme file.
}

impl OperatingMode {
    /// Converts an `usize` value into an `Option<Self>` value.
    ///
    /// # Arguments
    ///
    /// * `i` - The `usize` value to convert.
    ///
    /// # Returns
    ///
    /// An `Option<Self>` value representing the converted enum variant.
    /// If `i` is 1, returns `Some(Self::SetNewLicense)`.
    /// If `i` is 2, returns `Some(Self::AppendLicense)`.
    /// If `i` is 3, returns `Some(Self::LicenseReplace)`.
    /// If `i` is 4, returns `Some(Self::ShowAllGitDirs)`.
    /// If `i` is 5, returns `Some(Self::Unlicense)`.
    /// Otherwise, returns `None`.
    pub fn from_usize(i: usize) -> Option<Self> {
        match i {
            1 => { Some(Self::SetNewLicense) }
            2 => { Some(Self::AppendLicense) }
            3 => { Some(Self::LicenseReplace) }
            4 => { Some(Self::ShowAllGitDirs) }
            5 => { Some(Self::Unlicense) }
            _ => None
        }
    }
}
