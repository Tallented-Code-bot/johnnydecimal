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
    /// Show part or all of a Johnny Decimal system
    Show {
        /// The thing to show.
        ///
        /// This should be part or all of a Johnny Decimal number.
        /// - PRO.AC.ID or AC.ID
        /// - PRO
        /// AC or PRO.AC
        ///
        /// If this is not given, or something other than acceptable values is given,
        /// the whole Johnny Decimal system is shown.
        item: Option<String>,
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
    /// Add a Johnny Decimal number to the system
    Add {
        /// The category to add the number to
        category: String,
        /// The title of the number
        title: String,
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
        Subcommand::Show { item: term } => {
            let system = print_error(get_system())?;
            let output = print_error(system.display(term))?;
            println!("{}", output);
        }
        Subcommand::Display => {
            let system = print_error(get_system())?;
            println!("{}", system);
        }
        Subcommand::Cd { term } => match go_to_jd(term) {
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
        Subcommand::Add { category, title } => {
            let mut system = print_error(get_system())?;

            print_error(system.add_id_from_str(category, title))?;

            print_error(write_index(system))?;
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
    fs::write(
        &filepath,
        ron::ser::to_string_pretty(&system, ron::ser::PrettyConfig::new()).unwrap(),
    )
    .expect("Could not write file");

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
        InitShell::Bash => r#"
function j(){
    cd $(jd cd "$@")
}
"#
        .to_string(),
        InitShell::Zsh => r#"
function j(){
    cd $(jd cd "$@")
}

"#
        .to_string(),
        _ => {
            format!("{} {}","Error:".magenta(),"Unsupported shell.  The list of supported shells is currently bash, zsh, and fish.  More will be added eventually.")
            // TODO Change error message when adding shells
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

fn go_to_jd(input: String) -> Result<(), String> {
    let system = get_system()?;
    let jd_term = JdNumber::try_from(input)?;
    let jd = system.get_id(jd_term)?;

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

fn write_index(system: System) -> Result<(), &'static str> {
    let mut path = env::current_dir().unwrap();
    let file = path::Path::new(".JdIndex");

    loop {
        path.push(file);

        if path.is_file() {
            break;
        }

        if !(path.pop() && path.pop()) {
            return Err("Could not find index file to write to.");
        }
    }

    match fs::write(
        path,
        ron::ser::to_string_pretty(&system, ron::ser::PrettyConfig::new()).unwrap(),
    ) {
        Ok(_result) => Ok(()),
        Err(_err) => Err("Cannot write to file."),
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
fn _search(search: &str) -> Result<JdNumber, &str> {
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
