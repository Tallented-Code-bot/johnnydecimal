use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{env, fs, path};
use walkdir::{DirEntry, WalkDir};

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    /// Index an existing Johnny Decimal system
    Index {
        #[clap(parse(from_os_str))]
        path: path::PathBuf,
    },
    Search {
        term: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.action {
        Action::Index { path } => {
            index(path);
        }
        Action::Search { term } => match search(&term) {
            Ok(result) => println!("{}", result),
            Err(message) => println!("{}", message),
        },
    }
}

/// Create an index for a johnnydecimal system
fn index(filepath: path::PathBuf) {
    let mut system = System::new(); // create an empty JD system.

    let walker = WalkDir::new(&filepath).into_iter(); // Create a new filewalker.
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        //Walk through every file and directory:
        let path = entry.as_ref().unwrap().path().to_str().unwrap();
        let jd_number: JdNumber = match JdNumber::try_from(String::from(path)) {
            //check if it is a JD number,
            Ok(number) => number,
            Err(_err) => continue, //and if it is not, go to the next item.
        };

        println!("Indexing {}", jd_number);

        system.add_id(jd_number, "Hello".to_string());
    }

    fs::write(
        format!("{}.JdIndex", filepath.to_str().unwrap()),
        ron::to_string(&system).unwrap(),
    )
    .expect("Could not write file");
    println!(
        "Index has been written to {}.JdIndex",
        filepath.to_str().unwrap()
    );
}

/// Find the jd index file
// taken from https://codereview.stackexchange.com/questions/236743/find-a-file-in-current-or-parent-directories
fn find_index() -> Option<String> {
    let mut path = env::current_dir().unwrap();
    let file = path::Path::new(".JdIndex");

    loop {
        path.push(file);

        if path.is_file() {
            break Some(fs::read_to_string(path).unwrap());
        }

        if !(path.pop() && path.pop()) {
            break None;
        }
    }
}

/// Search for a johnny decimal number.
fn search(search: &str) -> Result<String, &str> {
    let search_number = match JdNumber::try_from(format!("{}_", search)) {
        Ok(jd) => jd,
        Err(_) => return Err("Search term was not a valid Johnny Decimal number"),
    };

    let index = match find_index() {
        Some(index) => index,
        None => return Err("Not in a valid Johnny Decimal system."),
    };

    let system: System = match ron::from_str(&index) {
        Ok(x) => x,
        Err(_) => return Err("Cannot read index file."),
    };

    // Regular linear search.  Sometime I might want to change this to a binary search.
    for jd in system.id {
        if jd.category == search_number.category
            && jd.id == search_number.id
            && jd.project == search_number.project
        {
            return Ok(jd.to_string());
        }
    }
    return Err("Cannot find number");
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

#[derive(Debug, Serialize, Deserialize)]
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

    fn new() -> Self {
        System {
            root: String::from("this_is_a_root"),
            projects: Vec::new(),
            id: Vec::new(),
            title: Vec::new(),
        }
    }
}

impl std::fmt::Display for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

/// A Johnny.Decimal number.
///
/// Can be either `PRO.AC.ID` or `AC.ID`.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
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
            label,
        });
    }
}

impl TryFrom<String> for JdNumber {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let re = Regex::new(r"((?:\d{3}\.)?\d{2}\.\d{2})(\D.*$)").unwrap();
        let captures = match re.captures(&value) {
            Some(x) => x,
            None => return Err(()),
        };

        let numbers = captures.get(1).unwrap().as_str().split(".").into_iter();
        let label = captures.get(2).unwrap().as_str();

        //check that there are periods in the number.
        //if !value.contains("."){
        //    return Err(());
        //}

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
                write!(
                    f,
                    "{:0>3}.{:0>2}.{:0>2}{}",
                    project, self.category, self.id, self.label
                )
            }
            None => {
                write!(f, "{:0>2}.{:0>2}{}", self.category, self.id, self.label)
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
        assert_ne!(
            JdNumber::try_from("05.02_label".to_string())
                .unwrap()
                .to_string(),
            "5.2_label"
        );
    }
}
