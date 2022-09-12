use clap::Parser;
use colored::Colorize;
use libc;
use regex::Regex;
use std::path::PathBuf;
use std::{env, fs, path};
use walkdir::{DirEntry, WalkDir};

pub mod jdnumber;
pub mod system;

use jdnumber::JdNumber;
use system::System;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// Index an existing Johnny Decimal system
    Index {
        #[clap(parse(from_os_str))]
        path: path::PathBuf,
    },
    Show {
        term: String,
    },
    Display,
    Cd {
        term: String,
    },
    List,
    Init {
        shell: InitShell,
    },
    Search {
        term: Option<String>,
    },
}

#[derive(Debug, Parser)]
enum InitShell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    Posix,
    Powershell,
    Xonsh,
    Zsh,
}

impl std::str::FromStr for InitShell {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bash" => Ok(InitShell::Bash),
            "elvish" => Ok(InitShell::Elvish),
            "fish" => Ok(InitShell::Fish),
            "nushell" => Ok(InitShell::Nushell),
            "posix" => Ok(InitShell::Posix),
            "powershell" => Ok(InitShell::Powershell),
            "xonsh" => Ok(InitShell::Xonsh),
            "zsh" => Ok(InitShell::Zsh),
            _ => Err(String::from("unknown shell.")),
        }
    }
}

fn main() -> Result<(), ()> {
    let cli = Cli::parse();
    // let system;
    // match get_system() {
    //     Ok(sys) => system = sys,
    //     Err(message) => {
    //         println!("{} {}", "Error:".magenta(), message);
    //         return;
    //     }
    // };

    //return print_error(system.search("hi"));

    match cli.subcommand {
        Subcommand::Index { path } => {
            index(path);
        }
        Subcommand::Show { term } => {
            let system = print_error(get_system())?;
            let output = print_error(system.show(&term))?;
            println!("{}", output);
        }
        Subcommand::Display => {
            let system = print_error(get_system())?;
            println!("{}", system);
        }
        Subcommand::Cd { term } => match go_to_jd(&term) {
            Ok(_) => {}
            Err(message) => println!("{} {}", "Error:".magenta(), message),
        },
        Subcommand::List => {
            let system = print_error(get_system())?;

            for jd_number in system.id {
                println!("{}", jd_number);
            }
        }
        Subcommand::Init { shell } => init(shell),
        Subcommand::Search { term } => {
            match term {
                Some(_x) => {}
                None => {}
            };
            println!("hi");
        }
    }

    Ok(())
}

/// Print an error message
fn print_error<T>(input: Result<T, &str>) -> Result<T, ()> {
    match input {
        Ok(result) => Ok(result),
        Err(message) => {
            println!("{} {}", "Error:".magenta(), message);
            Err(())
        }
    }
}

/// Create an index for a johnnydecimal system
fn index(mut filepath: path::PathBuf) {
    let mut system = System::new(filepath.clone().canonicalize().unwrap()); // create an empty JD system.

    let walker = WalkDir::new(&filepath).into_iter(); // Create a new filewalker.
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        //Walk through every file and directory:

        if entry.as_ref().unwrap().file_type().is_file() {
            continue;
        }

        let path = entry.as_ref().unwrap().path();
        let jd_number: JdNumber = match JdNumber::try_from(PathBuf::from(path)) {
            //check if it is a JD number,
            Ok(number) => number,
            Err(_err) => continue, //and if it is not, go to the next item.
        };

        println!("{} {}", "Indexing".green(), jd_number);

        match system.add_id(jd_number) {
            Ok(_) => {}
            Err(x) => {
                println!("{} {}", "Error:".magenta(), x)
            }
        }
    }

    filepath.push(".JdIndex");
    fs::write(&filepath, ron::to_string(&system).unwrap()).expect("Could not write file");
    println!("Index has been written to {}", filepath.display());
}

fn init(shell: InitShell) {
    // use the libc c interface to check if stdout is a tty or a pipe.
    let istty = unsafe { libc::isatty(libc::STDOUT_FILENO as i32) } != 0;

    let text = match shell {
        InitShell::Fish => "
function j
    pushd $(jd cd $argv)
end"
        .to_string(),
        _ => {
            format!("{} {}","Error:".magenta(),"Unsupported shell.  The list of supported shells is currently only fish.  More will be added eventually.")
        }
    };

    // if it is a tty, print a warning message
    if istty {
        println!("{} This command is not meant to be used in the terminal.  Use it in your shell config to set up the ability to cd to Johnny Decimal numbers.

Here is what would normally be output:
{}", "Warning:".yellow(),text);
    } else {
        println!("{}", text);
    }
}

fn go_to_jd(jd: &str) -> Result<(), &str> {
    let system = get_system()?;
    let jd = search(jd)?;

    // let path = format!(
    //     "{}/{}",
    //     system.path.to_str().unwrap(),
    //     jd.get_relative_path()
    // );
    let mut path = system.path;
    path.push(jd.get_relative_path());

    println!("{}", path.display());

    return Ok(());
    // match env::set_current_dir(path) {
    //     Ok(_) => return Ok(()),
    //     Err(_) => return Err("Unable to change to the correct directory."),
    // };
}

// fn display_overview() -> Result<String, &'static str> {
//     let system = get_system()?;

//     return Ok(system.to_string());
// }

// fn list() -> Result<(), &'static str> {
//     let system = get_system()?;

//     for jd_number in system.id {
//         println!("{}", jd_number);
//     }

//     return Ok(());
// }

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

fn get_system() -> Result<System, &'static str> {
    let index = match find_index() {
        Some(index) => index,
        None => return Err("Not in a valid Johnny Decimal system"),
    };

    let system: System = match ron::from_str(&index) {
        Ok(x) => x,
        Err(_) => return Err("Cannot read index file."),
    };
    return Ok(system);
}

/// Search for a johnny decimal number.
fn search(search: &str) -> Result<JdNumber, &str> {
    let re = Regex::new(r"(\d{3})?\.?(\d{2})\.(\d{2})").unwrap();

    let captures = match re.captures(search) {
        Some(x) => x,
        None => return Err("kj"),
    };

    let category: u32 = captures.get(2).unwrap().as_str().parse().unwrap();
    let id: u32 = captures.get(3).unwrap().as_str().parse().unwrap();
    let project = match captures.get(1) {
        Some(x) => Some(x.as_str().parse::<u32>().unwrap()),
        None => None,
    };

    let to_find = JdNumber::new(
        "cat",
        "area",
        category,
        id,
        project,
        None,
        "label".to_string(),
        PathBuf::new(),
    )
    .unwrap();

    let system = get_system()?;

    return match system.id.binary_search(&to_find) {
        Ok(index) => Ok(system.id[index].clone()),
        Err(_) => Err("Cannot find number"),
    };

    // // Regular linear search.  Sometime I might want to change this to a binary search.
    // for jd in system.id {
    //     if jd.category == category && jd.id == id && jd.project == project {
    //         return Ok(jd);
    //     }
    // }
    // return Err("Cannot find number");
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

// v----------------------- TESTS----------------v
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::JdNumber;
    use colored::Colorize;

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
    fn test_jd_from_string() {
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
    fn test_jd_display() {
        assert_eq!(
            JdNumber::try_from(PathBuf::from("20-29_area/20_category/20.35_label"))
                .unwrap()
                .to_string(),
            format!("{}.{}{}", "20".red(), "35".red(), "_label".red()) //"20.35_label"
        );
        assert_eq!(
            JdNumber::try_from(PathBuf::from(
                "300-399/project_area/352_project/40-49_area/45_category/352.45.30_label"
            ))
            .unwrap()
            .to_string(),
            format!(
                "{}.{}.{}{}",
                "352".red(),
                "45".red(),
                "30".red(),
                "_label".red()
            ) //"352.45.30_label"
        );
        assert_ne!(
            PathBuf::try_from("00-09_area/05_category/05.02_label".to_string())
                .unwrap()
                .display()
                .to_string(),
            format!("{}.{}{}", "5".red(), "2".red(), "_label".red()) //"5.2_label"
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
}
