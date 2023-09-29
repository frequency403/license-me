use std::fmt::{Display, Formatter};
use std::path::{MAIN_SEPARATOR, Path};

static README_VARIANTS: [&str;6] = ["README", "README.md", "README.MD", "readme.md", "Readme.md", "Readme.MD"];
static LICENSE_VARIANTS: [&str;3] = ["LICENSE", "license", "License"];

#[derive(Clone, Eq, PartialEq)]
pub struct GitDir {
    pub(crate)path: String,
    pub(crate) has_areadme: bool,
    pub(crate) readme_path: Option<String>,
    pub(crate) has_alicense: bool,
    pub(crate) license_path: Option<String>,
    pub(crate)project_title: String
}

impl GitDir {
    pub fn init(path: String) -> Self {
        let clean_path = path.replace(format!("{}.git", MAIN_SEPARATOR).as_str(), "");
        let project_title = clean_path.split(MAIN_SEPARATOR).last().unwrap().to_string();

        let mut has_readme = false;
        let mut has_license = false;
        let mut readme_path: Option<String> = None;
        let mut license_path: Option<String> = None;

        for variant in README_VARIANTS {
            if !has_readme {
                let temp_pth = format!("{}{}{}", clean_path, MAIN_SEPARATOR, variant);
                has_readme = Path::new(temp_pth.clone().as_str()).exists();
                readme_path = if has_readme { Some(temp_pth) } else { None };
            }
        }

        for variant in LICENSE_VARIANTS {
            if !has_license {
                let temp_pth = format!("{}{}{}", &clean_path, MAIN_SEPARATOR, variant);
                has_license = Path::new(temp_pth.clone().as_str()).exists();
                license_path = if has_license { Some(temp_pth) } else { None };
            }
        }

        Self {

            path: clean_path,
            has_areadme: has_readme,
            readme_path,
            has_alicense: has_license,
            license_path,
            project_title
        }
    }
}

impl Display for GitDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n{}\nPath: {}\nLicense: {} | Readme: {}", self.project_title,self.path, self.has_alicense, self.has_areadme)
    }
}
