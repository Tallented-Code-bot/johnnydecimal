use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    cmp,
    path::{self, PathBuf},
};

/// A location of a Johnny Decimal number.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Location {
    Path(path::PathBuf),
}

/// A Johnny.Decimal number.
///
/// Can be either `PRO.AC.ID` or `AC.ID`.
/// In path form a Johnny Decimal number looks something like this:
/// `20-29_area_label/25_category_label/25.21_jd_label`
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JdNumber {
    /// The project number, if it exists
    pub project: Option<u32>,
    /// The project label, for example 101**_project_1**
    pub project_label: Option<String>,
    /// The category, between 0 and 99.
    pub category: u32,
    /// The id, between 0 and 99.
    pub id: u32,
    /// The label, for example 50.42**_this_is_the_label**.
    pub label: String,
    /// The area label:
    pub area_label: String,
    /// The category label
    pub category_label: String,
    /// The path of the JD number relative to the system root.
    pub path: Location,
}
impl JdNumber {
    /// Create a new JD number, with some error checking.
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

    /// Get the area+label of a JD number.
    ///
    /// This returns a string in the format
    /// `50-59_area_label`
    pub fn get_area(&self) -> String {
        format!(
            "{area}0-{area}9{}",
            self.area_label,
            area = self.category.to_string().chars().nth(0).unwrap()
        )
    }

    /// Get the relative path of a JD number.
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

    /// Check if two Johnny Decimal numbers are exactly equal.
    ///
    /// This include the numbers, and **all** fields.  
    pub fn check_exactly_equal(jd1: JdNumber, jd2: JdNumber) -> bool {
        println!("{:?}\n{:?}\n\n", jd1, jd2);
        return jd1.project == jd2.project
            && jd1.project_label == jd2.project_label
            && jd1.category == jd2.category
            && jd1.id == jd2.id
            && jd1.label == jd2.label
            && jd1.area_label == jd2.area_label
            && jd1.category_label == jd2.category_label
            && jd1.path == jd2.path;
    }
}

/// Create a johnny decimal number from a path.
impl TryFrom<PathBuf> for JdNumber {
    type Error = &'static str;
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
        let category_ex = Regex::new(r"^(\d\d)([^0-9.].*)$").expect("Hardcoded regex is valid.");
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
        let mut area: Option<(u32, u32)> = None;
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
                    area = Some((
                        caps.get(1).unwrap().as_str().parse().unwrap(),
                        caps.get(2).unwrap().as_str().parse().unwrap(),
                    ));
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
            return Err("");
        }

        if category != jd_category {
            return Err("");
        }

        // If the first area number is not a multiple
        // of ten, error.
        if area.ok_or("")?.0 % 10 != 0 {
            return Err("First area number is not a multiple of 10.");
        }
        // if the second area number is not 9 more than the first one,
        // error.
        if area.ok_or("")?.1 != area.ok_or("")?.0 + 9 {
            return Err("Second area number is not 9 more than the first number.");
        }

        return match JdNumber::new(
            area_name.ok_or("Could not find area name")?,
            category_name.ok_or("Could not find category name")?,
            category.ok_or("Could not find category")?,
            jd_id.ok_or("Could not find id")?,
            project,
            project_name.map(|p| p.to_string()),
            jd_name.ok_or("Could not find JD name")?.to_string(),
            path_value.clone(),
        ) {
            Ok(value) => Ok(value),
            Err(_) => Err("Could not create JD number"),
        };
    }
}

impl TryFrom<String> for JdNumber {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, &'static str> {
        // PRO.AC.ID or AC.ID
        let ex = Regex::new(r"^(\d\d\d)?\.?(\d\d)\.(\d\d)$").expect("Hardcoded regex is valid.");
        let project: Option<u32>;
        let category: u32;
        let id: u32;

        match ex.captures(&value) {
            Some(caps) => {
                project = caps.get(1).map(|v| v.as_str().parse().unwrap());
                category = caps
                    .get(2)
                    .map(|v| v.as_str().parse().unwrap())
                    .ok_or("Could not get project.")?;
                id = caps
                    .get(3)
                    .map(|v| v.as_str().parse().unwrap())
                    .ok_or("Could not get id.")?;
            }
            None => return Err("Regex did not match"),
        };

        return match JdNumber::new(
            "",
            "",
            category,
            id,
            project,
            None,
            "label".to_string(),
            PathBuf::new(),
        ) {
            Ok(jd) => Ok(jd),
            Err(_err) => Err("Could not create JD"),
        };
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
        if self.project.is_some() && other.project.is_some() {
            // check which project is greater
            if self.project.unwrap() > other.project.unwrap() {
                return cmp::Ordering::Greater;
            }
            if self.project.unwrap() < other.project.unwrap() {
                return cmp::Ordering::Less;
            }
        } else if self.project.is_some() && other.project.is_none() {
            // some project trumps none
            return cmp::Ordering::Greater;
        } else if self.project.is_none() && other.project.is_some() {
            return cmp::Ordering::Less;
        }

        // projects are guaranteed to be equal now.
        if self.category > other.category {
            return cmp::Ordering::Greater;
        }
        if self.category < other.category {
            return cmp::Ordering::Less;
        }
        // category has to be equal.
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
        if self.project.is_some() && other.project.is_some() {
            if self.project.unwrap() > other.project.unwrap() {
                return Some(cmp::Ordering::Greater);
            }
            if self.project.unwrap() < other.project.unwrap() {
                return Some(cmp::Ordering::Less);
            }
        } else if self.project.is_some() && other.project.is_none() {
            return Some(cmp::Ordering::Greater);
        } else if self.project.is_none() && other.project.is_some() {
            return Some(cmp::Ordering::Less);
        }

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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::JdNumber;

    use crate::jdnumber::Location;

    #[test]
    fn test_jd_creation() {
        assert!(JdNumber::new(
            "",
            "",
            100,
            524,
            None,
            None,
            "fsd".to_string(),
            PathBuf::new()
        )
        .is_err());
        assert!(JdNumber::new(
            "fsd",
            "fsd",
            43,
            23,
            None,
            None,
            "sdf".to_string(),
            PathBuf::new()
        )
        .is_ok());
        assert!(JdNumber::new(
            "slfd",
            "sdd",
            100,
            52,
            Some(402),
            Some("hi".to_string()),
            "_goodbye".to_string(),
            PathBuf::new()
        )
        .is_err());
        assert!(JdNumber::new(
            "hi",
            "bye",
            52,
            24,
            Some(2542),
            Some("label".to_string()),
            " hello".to_string(),
            PathBuf::new()
        )
        .is_err());
    }
    #[test]
    fn test_jd_from_path() {
        assert_eq!(
            JdNumber::try_from(PathBuf::from("20-29_testing/20_good_testing/20.35_test")).unwrap(),
            JdNumber {
                category: 20,
                id: 35,
                project: None,
                project_label: None,
                label: String::from("_test"),
                category_label: String::from("_good_testing"),
                area_label: String::from("_testing"),
                path: Location::Path(PathBuf::from("20-29_testing/20_good_testing/20.35_test"))
            }
        );
        assert_eq!(
            JdNumber::try_from(PathBuf::from("50-59_hi/50_bye/50.32_label")).unwrap(),
            JdNumber {
                category: 50,
                id: 32,
                project: None,
                project_label: None,
                label: String::from("_label"),
                area_label: String::from("_hi"),
                category_label: String::from("_bye"),
                path: Location::Path(PathBuf::from("50-59_hi/50_bye/50.32_label"))
            }
        );
        assert_eq!(
            JdNumber::try_from(PathBuf::from(
                "100-199_school/102_grade-10/20-29_RHS/22-ap_biology/102.22.02_oreo_project"
            ))
            .unwrap(),
            JdNumber {
                category: 22,
                id: 02,
                project: Some(102),
                project_label: Some("_grade-10".to_string()),
                label: String::from("_oreo_project"),
                category_label: String::from("-ap_biology"),
                area_label: String::from("_RHS"),
                path: Location::Path(PathBuf::from(
                    "100-199_school/102_grade-10/20-29_RHS/22-ap_biology/102.22.02_oreo_project"
                ))
            }
        );

        assert!(JdNumber::check_exactly_equal(
            JdNumber::try_from(PathBuf::from("10-19_finance/12_payroll/12.02_a_payroll")).unwrap(),
            JdNumber {
                category: 12,
                id: 02,
                project: None,
                project_label: None,
                label: String::from("_a_payroll"),
                category_label: String::from("_payroll"),
                area_label: String::from("_finance"),
                path: Location::Path(PathBuf::from("10-19_finance/12_payroll/12.02_a_payroll"))
            }
        ));
        // assert_eq!(
        //     JdNumber::try_from(String::from("60-69/62/423.62.21 hi")).unwrap(),
        //     JdNumber {
        //         category: 62,
        //         id: 21,
        //         project: Some(423),
        //         label: String::from(" hi"),
        //         area_label: String::new(),
        //         category_label: String::new(),
        //         path: Location::Path(PathBuf::from("60-69/62/423.62.21 hi"))
        //     }
        // );
        assert!(JdNumber::try_from(PathBuf::from("5032")).is_err());
        assert!(JdNumber::try_from(PathBuf::from("hi.by")).is_err());
        assert!(JdNumber::try_from(PathBuf::from("324.502")).is_err());
        assert!(JdNumber::try_from(PathBuf::from("3006.243.306")).is_err());
        assert!(JdNumber::try_from(PathBuf::from("20.43")).is_err());
        //assert!(JdNumber::try_from(String::from("500.42.31")).is_err());
    }

    #[test]
    fn test_jd_from_string() {
        // Test PRO.AC.ID
        assert_eq!(
            JdNumber::try_from(String::from("192.13.42")).unwrap(),
            JdNumber::new(
                "",
                "",
                13,
                42,
                Some(192),
                None,
                "j".to_string(),
                PathBuf::new()
            )
            .unwrap()
        );

        // test AC.ID
        assert_eq!(
            JdNumber::try_from(String::from("50.42")).unwrap(),
            JdNumber::new("", "", 50, 42, None, None, "l".to_string(), PathBuf::new()).unwrap()
        );

        // test empty string
        assert!(JdNumber::try_from(String::from("")).is_err());

        // test giberish
        assert!(JdNumber::try_from(String::from("this_is-some|giberish!")).is_err());
    }

    #[test]
    fn test_jd_display() {
        assert_eq!(
            JdNumber::try_from(PathBuf::from("20-29_area/20_category/20.35_label"))
                .unwrap()
                .to_string(),
            format!("{}.{}{}", "20", "35", "_label") //"20.35_label"
        );
        assert_eq!(
            JdNumber::try_from(PathBuf::from(
                "300-399/project_area/352_project/40-49_area/45_category/352.45.30_label"
            ))
            .unwrap()
            .to_string(),
            format!("{}.{}.{}{}", "352", "45", "30", "_label") //"352.45.30_label"
        );
        assert_ne!(
            PathBuf::try_from("00-09_area/05_category/05.02_label".to_string())
                .unwrap()
                .display()
                .to_string(),
            format!("{}.{}{}", "5", "2", "_label") //"5.2_label"
        );
    }

    #[test]
    fn test_jd_equality() {
        let jd_1 = JdNumber::new(
            "area_lab",
            "cat_lab",
            50,
            32,
            None,
            None,
            "this_lab".to_string(),
            PathBuf::new(),
        )
        .unwrap();
        let jd_2 = JdNumber::new(
            "diff_area",
            "diff_cat",
            50,
            32,
            None,
            None,
            "diflab".to_string(),
            PathBuf::new(),
        )
        .unwrap();
        assert_eq!(jd_1, jd_2);
        let jd_3 = JdNumber::new(
            "arealabel",
            "catlabel",
            40,
            33,
            None,
            None,
            "a_label".to_string(),
            PathBuf::new(),
        )
        .unwrap();
        assert_ne!(jd_1, jd_3);
        assert_ne!(jd_2, jd_3);
    }

    #[test]
    fn test_clone() {
        let jd_1 = JdNumber::new(
            "area_label",
            "cat_label",
            50,
            32,
            None,
            None,
            "here".to_string(),
            PathBuf::new(),
        )
        .unwrap();

        assert_eq!(jd_1, jd_1.clone());
    }

    #[test]
    fn test_exactly_equal() {
        let jd_1 = JdNumber::new(
            "area_label",
            "cat_label",
            50,
            32,
            None,
            None,
            "here".to_string(),
            PathBuf::new(),
        )
        .unwrap();

        let jd_2 = JdNumber::new(
            "area_label",
            "cat_label",
            50,
            32,
            None,
            None,
            "here".to_string(),
            PathBuf::new(),
        )
        .unwrap();

        assert!(JdNumber::check_exactly_equal(jd_1, jd_2.clone()));

        let jd_3 = JdNumber::new(
            "area_2_label",
            "cat_2_label",
            60,
            32,
            None,
            None,
            "here".to_string(),
            PathBuf::new(),
        )
        .unwrap();
        assert!(!JdNumber::check_exactly_equal(jd_2, jd_3));
    }
    #[test]
    fn test_ord() {
        // test project inequality
        let jd_1 = JdNumber::try_from("102.22.05".to_string());
        let jd_2 = JdNumber::try_from("101.22.05".to_string());
        assert!(jd_1 > jd_2);
        assert!(jd_2 < jd_1);

        // test category inequality
        let jd_3 = JdNumber::try_from("100.30.05".to_string());
        let jd_4 = JdNumber::try_from("100.31.05".to_string());
        assert!(jd_4 > jd_3);
        assert!(jd_3 < jd_4);

        // test id inequality
        let jd_5 = JdNumber::try_from("300.50.03".to_string());
        let jd_6 = JdNumber::try_from("300.50.02".to_string());
        assert!(jd_5 > jd_6);
        assert!(jd_6 < jd_5);

        // test equality
        let jd_7 = JdNumber::try_from("502.43.10".to_string());
        let jd_8 = JdNumber::try_from("502.43.10".to_string());
        assert_eq!(jd_7, jd_8);
        assert_eq!(jd_8, jd_7);
    }
}
