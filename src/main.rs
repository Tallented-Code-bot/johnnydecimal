use regex::Regex;
use walkdir::{DirEntry, WalkDir};

fn main() {
    index();
}

/// Create an index for a johnnydecimal system
fn index() {
    let re = Regex::new(r"\d{2}\.\d{2}.*$").unwrap();

    let walker = WalkDir::new("/home/gitpod/jd/").into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        if re.is_match(entry.as_ref().unwrap().path().to_str().unwrap()) {
            //println!("match: {}",entry.unwrap().path().display());
            println!(
                "{}",
                re.find(entry.unwrap().path().to_str().unwrap())
                    .unwrap()
                    .as_str()
            )
        }
        //println!("{}",entry.unwrap().path().display());
    }
}

/// Checks if a given file or directory is hidden.
///
/// Taken from https://docs.rs/walkdir/latest/walkdir/
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

#[derive(Debug)]
struct System {
    root: String,
    projects: Vec<String>,
    //area:Vec<String>,
    //category:Vec<String>,
    /// Looks like 100.24.34
    id: Vec<JdNumber>,
    title: Vec<String>,
}

impl System {
    /// Add an id to the system
    fn add_id(&mut self, id: JdNumber, name: String) {
        self.id.push(id);
        self.title.push(name);
    }
}

/// A Johnny.Decimal number.
///
/// Can be either `PRO.AC.ID` or `AC.ID`.
#[derive(PartialEq, Debug)]
struct JdNumber {
    project: Option<u32>,
    category: u32,
    id: u32,
    label: String,
}
impl JdNumber {
    fn new(category: u32, id: u32, project: Option<u32>, label: String) -> Result<Self, ()> {
        // If the area or category are too long, return none
        if category > 99 || id > 99 {
            return Err(());
        }

        match project {
            Some(project) => {
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
            label,
        });
    }
}

impl TryFrom<String> for JdNumber {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // regex:
        //
        // ----------  (\d{3}\.)?\d{2}\.\d{2}   -----------
        // ((?:\d{3}\.)?\d{2}\.\d{2})(\D.*$)
        //
        // (?<=\d{2}.\d{2})\D.*$
        //
        println!("value:{}", value);
        let re = Regex::new(r"((?:\d{3}\.)?\d{2}\.\d{2})(\D.*$)").unwrap();
        let captures = match re.captures(&value) {
            Some(x) => x,
            None => return Err(()),
        };

        let all_captures = captures.iter();
        for i in all_captures {
            println!(
                "capture:{}",
                match i {
                    Some(x) => {
                        x.as_str()
                    }
                    None => {
                        "None"
                    }
                }
            )
        }

        println!(
            "value:{}, capture:{}",
            value,
            captures.get(0).unwrap().as_str()
        );

        let numbers = captures.get(1).unwrap().as_str().split(".").into_iter();
        let label = captures.get(2).unwrap().as_str();

        //check that there are periods in the number.
        //if !value.contains("."){
        //    return Err(());
        //}

        //let numbers=value.split(".").into_iter();
        let mut new_numbers: Vec<u32> = Vec::new();

        // for each string in the generated list, parse it into
        // a number.  If it does not parse, error.
        for number in numbers {
            match number.parse() {
                Ok(x) => new_numbers.push(x),
                Err(_error) => return Err(()),
            };
        }

        if new_numbers.len() == 3 {
            return JdNumber::new(
                new_numbers[1],
                new_numbers[2],
                Some(new_numbers[0]),
                label.to_string(),
            );
        } else {
            return JdNumber::new(new_numbers[0], new_numbers[1], None, label.to_string());
        }
    }
}

impl std::fmt::Display for JdNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.project {
            Some(project) => {
                write!(f, "{}.{}.{}{}", project, self.category, self.id, self.label)
            }
            None => {
                write!(f, "{}.{}{}", self.category, self.id, self.label)
            }
        }
    }
}

// v----------------------- TESTS----------------v
#[cfg(test)]
mod tests {
    use crate::JdNumber;

    #[test]
    fn test_jd_creation() {
        assert!(JdNumber::new(100, 524, None, "fsd".to_string()).is_err());
        assert!(JdNumber::new(43, 23, None, "sdf".to_string()).is_ok());
        assert!(JdNumber::new(100, 52, Some(402), "_goodbye".to_string()).is_err());
        assert!(JdNumber::new(52, 24, Some(2542), " hello".to_string()).is_err());
    }
    #[test]
    fn test_jd_from_string() {
        assert_eq!(
            JdNumber::try_from(String::from("20.35_test")).unwrap(),
            JdNumber {
                category: 20,
                id: 35,
                project: None,
                label: String::from("_test")
            }
        );
        assert_eq!(
            JdNumber::try_from(String::from("50.32_label")).unwrap(),
            JdNumber {
                category: 50,
                id: 32,
                project: None,
                label: String::from("_label")
            }
        );
        assert_eq!(
            JdNumber::try_from(String::from("423.62.21 hi")).unwrap(),
            JdNumber {
                category: 62,
                id: 21,
                project: Some(423),
                label: String::from(" hi")
            }
        );
        assert!(JdNumber::try_from(String::from("5032")).is_err());
        assert!(JdNumber::try_from(String::from("hi.by")).is_err());
        assert!(JdNumber::try_from(String::from("324.502")).is_err());
        assert!(JdNumber::try_from(String::from("3006.243.306")).is_err());
        assert!(JdNumber::try_from(String::from("20.43")).is_err());
        //assert!(JdNumber::try_from(String::from("500.42.31")).is_err());
    }
    #[test]
    fn test_jd_display() {
        assert_eq!(
            JdNumber::try_from(String::from("20.35_label"))
                .unwrap()
                .to_string(),
            "20.35_label"
        );
        assert_eq!(
            JdNumber::try_from(String::from("352.45.30_label"))
                .unwrap()
                .to_string(),
            "352.45.30_label"
        );
    }
}
