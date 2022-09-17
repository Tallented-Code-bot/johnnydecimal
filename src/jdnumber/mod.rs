use colored::Colorize;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    cmp,
    path::{self, PathBuf},
};

/// A location of a Johnny Decimal number.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Location {
    Path(path::PathBuf),
}

/// A Johnny.Decimal number.
///
/// Can be either `PRO.AC.ID` or `AC.ID`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JdNumber {
    pub project: Option<u32>,
    pub project_label: Option<String>,
    pub category: u32,
    pub id: u32,
    pub label: String,
    pub area_label: String,
    pub category_label: String,
    pub path: Location,
}
impl JdNumber {
    pub fn new(
        area_label: &str,
        category_label: &str,
        category: u32,
        id: u32,
        project: Option<u32>,
        project_label: Option<String>,
        label: String,
        path: PathBuf,
    ) -> Result<Self, ()> {
        // If the area or category are too long, return none
        if category > 99 || id > 99 {
            return Err(());
        }

        match project {
            Some(project) => {
                // If the project has more than 3 digits, error.
                if project > 999 {
                    return Err(());
                }
            }
            None => {}
        }

        return Ok(JdNumber {
            category,
            id,
            project,
            project_label,
            label,
            area_label: area_label.to_string(),
            category_label: category_label.to_string(),
            path: Location::Path(path),
        });
    }

    pub fn get_area(&self) -> String {
        format!(
            "{area}0-{area}9{}",
            self.area_label,
            area = self.category.to_string().chars().nth(0).unwrap()
        )
    }

    pub fn get_relative_path(&self) -> PathBuf {
        // format!(
        //     "{}/{:0>2}{}/{:0>2}.{:0>2}{}",
        //     self.get_area(),
        //     self.category,
        //     self.category_label,
        //     self.category,
        //     self.id,
        //     self.label
        // )

        let mut path = PathBuf::new();

        if self.project.is_some() && self.project_label.is_some() {
            path.push(format!(
                "{:0>3}{}",
                self.project.unwrap(),
                self.project_label.as_ref().unwrap()
            ))
        }

        path.push(self.get_area());
        path.push(format!(
            "{:0>2}{}",
            self.category,
            self.category_label.clone()
        ));
        // path.push(self.category_label.clone());

        if self.project.is_none() {
            path.push(format!(
                "{:0>2}.{:0>2}{}",
                self.category,
                self.id,
                self.label.clone()
            ));
        } else {
            path.push(format!(
                "{:0>3}.{:0>2}.{:0>2}{}",
                self.project.unwrap(),
                self.category,
                self.id,
                self.label.clone()
            ))
        }
        // path.push(format!("{:0>2}", self.id));
        // path.push(self.label.clone());
        path
    }
}

/// Create a johnny decimal number from a path.
impl TryFrom<PathBuf> for JdNumber {
    type Error = ();
    // 20-29_testing/20_good_testing/20.35_test/

    fn try_from(path_value: PathBuf) -> Result<Self, Self::Error> {
        //let path_value = PathBuf::from(&value);

        // TODO think about lazily compiling these regi.
        // See https://docs.rs/regex/latest/regex/#example-avoid-compiling-the-same-regex-in-a-loop

        // initialize regi (plural of regex!)
        let project_area_ex =
            Regex::new(r"^(\d\d\d)-(\d\d\d)(\D.*)$").expect("Hardcoded regex is valid.");
        let project_ex = Regex::new(r"^(\d\d\d)([^0-9.].*)$").expect("Hardcoded regex is valid.");
        let area_ex = Regex::new(r"^(\d\d)-(\d\d)(\D.*)$").expect("Hardcoded regex is valid.");
        let category_ex = Regex::new(r"^(\d\d)(\D.*)$").expect("Hardcoded regex is valid.");
        // let jd_ex =
        //     Regex::new(r"^(\d\d\d)?\.?(\d)(\d)\.(\d\d)(.*)$").expect("Hardcoded regex is valid.");
        let jd_ex = Regex::new(r"(?m)^(\d\d\d)?\.?(\d\d)\.(\d\d)(\D.*)$")
            .expect("Hardcoded regex is valid");

        // Initialize variable
        let mut _project_area: Option<(&str, &str)> = None;
        let mut _project_area_name: Option<&str> = None;
        let mut project_name: Option<&str> = None;
        let mut project: Option<u32> = None;
        let mut area_name: Option<&str> = None;
        let mut _area: Option<(&str, &str)> = None;
        let mut category: Option<u32> = None;
        let mut category_name: Option<&str> = None;
        let mut jd_project: Option<u32> = None;
        let mut _jd_area: Option<u32> = None;
        let mut jd_category: Option<u32> = None;
        let mut jd_id: Option<u32> = None;
        let mut jd_name: Option<&str> = None;

        // Extract all the components
        for component in path_value.components() {
            match project_area_ex.captures(component.as_os_str().to_str().unwrap()) {
                Some(caps) => {
                    _project_area =
                        Some((caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str()));
                    // project_area_name = Some(caps.get(3).unwrap().as_str());
                    _project_area_name = caps.get(3).map(|v| v.as_str());
                }
                None => {}
            }

            match project_ex.captures(component.as_os_str().to_str().unwrap()) {
                Some(caps) => {
                    // project_name = Some(caps.get(2).unwrap().as_str());
                    project_name = caps.get(2).map(|v| v.as_str());
                    // project = Some(caps.get(1).unwrap().as_str());
                    project = caps.get(1).map(|v| v.as_str().parse().unwrap());
                }
                None => {}
            }

            match area_ex.captures(component.as_os_str().to_str().unwrap()) {
                Some(caps) => {
                    _area = Some((caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str()));
                    // area_name = Some(caps.get(3).unwrap().as_str());
                    area_name = caps.get(3).map(|v| v.as_str());
                }
                None => {}
            }

            match category_ex.captures(component.as_os_str().to_str().unwrap()) {
                Some(caps) => {
                    category = caps.get(1).map(|v| v.as_str().parse().unwrap());
                    category_name = caps.get(2).map(|v| v.as_str());
                }
                None => {}
            }

            match jd_ex.captures(component.as_os_str().to_str().unwrap()) {
                Some(caps) => {
                    jd_project = caps.get(1).map(|v| v.as_str().parse().unwrap());
                    // jd_area = caps.get(2).map(|v| v.as_str().parse().unwrap());
                    jd_category = caps.get(2).map(|v| v.as_str().parse().unwrap());
                    jd_id = caps.get(3).map(|v| v.as_str().parse().unwrap());
                    jd_name = caps.get(4).map(|v| v.as_str());
                }
                None => {}
            }
        }

        if project != jd_project {
            return Err(());
        }

        if category != jd_category {
            return Err(());
        }

        return JdNumber::new(
            area_name.ok_or(())?,
            category_name.ok_or(())?,
            category.ok_or(())?,
            jd_id.ok_or(())?,
            project,
            project_name.map(|p| p.to_string()),
            jd_name.ok_or(())?.to_string(),
            path_value.clone(),
        );
    }
}

impl std::fmt::Display for JdNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.project {
            Some(project) => {
                write!(
                    f,
                    "{:0>3}.{:0>2}.{:0>2}{}",
                    project.to_string(),       //.red(),
                    self.category.to_string(), //.red(),
                    self.id.to_string(),       //.red(),
                    self.label,                //.red()
                )
            }

            None => {
                write!(
                    f,
                    "{:0>2}.{:0>2}{}",
                    self.category.to_string(), //.red(),
                    self.id.to_string(),       //.red(),
                    self.label,                //.red()
                )
            }
        }
    }
}

impl Ord for JdNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.category > other.category {
            return cmp::Ordering::Greater;
        }
        if self.category < other.category {
            return cmp::Ordering::Less;
        }
        //first number is equal
        if self.id > other.id {
            return cmp::Ordering::Greater;
        }
        if self.id < other.id {
            return cmp::Ordering::Less;
        }
        cmp::Ordering::Equal
    }
}

// Keep this implement block
impl Eq for JdNumber {}
impl PartialEq for JdNumber {
    fn eq(&self, other: &Self) -> bool {
        self.project == other.project && self.category == other.category && self.id == other.id
    }
}

impl PartialOrd for JdNumber {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        if self.category > other.category {
            return Some(cmp::Ordering::Greater);
        }
        if self.category < other.category {
            return Some(cmp::Ordering::Less);
        }

        if self.id > other.id {
            return Some(cmp::Ordering::Greater);
        }
        if self.id < other.id {
            return Some(cmp::Ordering::Less);
        }
        return Some(cmp::Ordering::Equal);
    }
}
