use crate::jdnumber::JdNumber;
use nom::{
    character::complete::{char, line_ending, multispace0, not_line_ending, one_of},
    combinator::{not, opt, recognize, value},
    multi::{count, many0, many1},
    sequence::{pair, separated_pair, terminated, tuple},
    IResult,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path;
use std::path::PathBuf;

/// A Johnny Decimal system.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct System {
    /// The root path of the Johnny Decimal system.
    pub path: path::PathBuf,
    /// A list of Johnny Decimal numbers.
    pub id: Vec<JdNumber>,
}

impl System {
    /// Add an id to the system.
    ///
    /// This adds an id to the system only if it is not a
    /// duplicate; otherwise, it returns `Err()`.
    pub fn add_id(&mut self, id: JdNumber) -> Result<(), &str> {
        match self.id.binary_search(&id) {
            Ok(_pos) => return Err("Element already exists."),
            Err(pos) => self.id.insert(pos, id),
        };
        return Ok(());
    }

    /// Create a new System.
    pub fn new(path: path::PathBuf) -> Self {
        System {
            path,
            id: Vec::new(),
        }
    }

    //DEPRECATED
    // Keep this for awhile, then delete it.

    // fn show(&self, search: &str) -> Result<JdNumber, &str> {
    //     let re = Regex::new(r"(\d{3})?\.?(\d{2})\.(\d{2})").unwrap();

    //     let captures = match re.captures(search) {
    //         Some(x) => x,
    //         None => return Err("Invalid search term.  Search term should be a valid JD number."),
    //     };

    //     let category: u32 = captures.get(2).unwrap().as_str().parse().unwrap();
    //     let id: u32 = captures.get(3).unwrap().as_str().parse().unwrap();
    //     let project = match captures.get(1) {
    //         Some(x) => Some(x.as_str().parse::<u32>().unwrap()),
    //         None => None,
    //     };

    //     let to_find = JdNumber::new(
    //         "cat",
    //         "area",
    //         category,
    //         id,
    //         project,
    //         Some("project_label".to_string()),
    //         "label".to_string(),
    //         PathBuf::new(),
    //     )
    //     .unwrap();

    //     return match self.id.binary_search(&to_find) {
    //         Ok(index) => Ok(self.id[index].clone()),
    //         Err(_) => Err("Cannot find number."),
    //     };
    // }

    /// Parse a Jd input.
    ///
    /// The input should be a Johnny Decimal number, or a partial Johnny Decimal number.
    /// The types of numbers can be:
    /// - PRO.AC.ID or AC.ID
    /// - PRO
    /// - AC/PRO.AC
    ///
    /// It returns a tuple of the project, category, and id.
    fn parse_jd_input(input: String) -> (Option<u32>, Option<u32>, Option<u32>) {
        // PRO.AC or AC
        let cat_ex = Regex::new(r"^(\d\d\d)?\.?(\d\d)$").expect("Hardcoded regex is valid.");
        // PRO
        let project_ex = Regex::new(r"^(\d\d\d)$").expect("Hardcoded regex is valid.");
        // PRO.AC.ID or AC.ID
        let jd_ex = Regex::new(r"^(\d\d\d)?\.?(\d\d)\.(\d\d)$").expect("Hardcoded regex is valid.");

        let mut project: Option<u32> = None;
        let mut category: Option<u32> = None;
        let mut id: Option<u32> = None;

        match cat_ex.captures(&input) {
            Some(caps) => {
                project = caps.get(1).map(|v| v.as_str().parse().unwrap());
                category = caps.get(2).map(|v| v.as_str().parse().unwrap());
            }
            None => {}
        };

        match project_ex.captures(&input) {
            Some(caps) => {
                project = caps.get(1).map(|v| v.as_str().parse().unwrap());
            }
            None => {}
        };

        match jd_ex.captures(&input) {
            Some(caps) => {
                project = caps.get(1).map(|v| v.as_str().parse().unwrap());
                category = caps.get(2).map(|v| v.as_str().parse().unwrap());
                id = caps.get(3).map(|v| v.as_str().parse().unwrap());
            }
            None => {}
        };

        return (project, category, id);
    }

    /// Display a johnny decimal system.
    ///
    /// The input should be a Johnny Decimal number, or a partial Johnny Decimal number.
    /// The types of numbers can be:
    /// - PRO.AC.ID or AC.ID
    /// - PRO
    /// - AC/PRO.AC
    ///
    /// For example,
    /// `display(Some(String::from("50.43")))` should display the Johnny Decimal
    /// `50.43`.
    ///
    /// If `None`, an empty string, or a string with some other giberish
    /// is input, the whole Johnny Decimal system will be displayed.
    pub fn display(&self, input: Option<String>) -> Result<String, &str> {
        // let mut project: Option<u32> = None;
        // let mut category: Option<u32> = None;
        // let mut id: Option<u32> = None;

        let (project, category, id) = System::parse_jd_input(input.unwrap_or("".to_string()));

        let mut jd_list: Vec<&JdNumber>;

        // if is a full decimal number, show it.
        if id.is_some() && category.is_some() {
            let to_find = JdNumber::new(
                "",
                "",
                category.expect("Has already been checked to be some."),
                id.expect("Has already been checked to be some."),
                project,
                None,
                None,
                String::new(),
                PathBuf::new(),
            )
            .expect("Manual jd number is valid");

            match self.id.binary_search(&to_find) {
                Ok(pos) => jd_list = vec![&self.id[pos]],
                Err(_pos) => return Err("Cannot find JD number."),
            };
            // otherwise, if it is a category, show it.
        } else if category.is_some() {
            jd_list = Vec::new();
            for item in &self.id {
                if item.category == category.unwrap() && item.project == project {
                    jd_list.push(&item);
                }
            }
        } else if project.is_some() {
            jd_list = Vec::new();
            for item in &self.id {
                if item.project == project {
                    jd_list.push(&item);
                }
            }
        } else {
            jd_list = Vec::new();
            for item in &self.id {
                jd_list.push(&item);
            }
        }

        // display the filtered numbers
        let mut output = String::new();
        let mut area_string = String::new();
        let mut category_string = String::new();
        let mut project_string: Option<String> = None;
        for i in jd_list {
            if i.project_label != project_string {
                project_string = i.project_label.clone();
                output.push_str(
                    format!(
                        "{}{}",
                        i.project.unwrap(),
                        &i.project_label.clone().unwrap_or(String::new())
                    )
                    .as_str(),
                );
                output.push_str("\n");
            }
            if i.area_label != area_string {
                area_string = i.area_label.clone();
                output.push_str(format!("  {}", &i.get_area().as_str()).as_str());
                output.push_str("\n");
            }
            if i.category_label != category_string {
                category_string = i.category_label.clone();
                output.push_str(format!("    {}{}", i.category, &i.category_label).as_str());
                output.push_str("\n");
            }
            output.push_str(format!("      {}", i.to_string()).as_str());
            output.push_str("\n");
        }

        return Ok(output);
    }

    /// Filter by PRO, AC, and/or ID.
    ///
    /// Given PRO, AC, and ID arguments, this returns
    /// all the JD numbers that match.  If a parameter is none
    /// it is not filtered by.
    fn filter_id(
        &self,
        project: Option<u32>,
        category: Option<u32>,
        id: Option<u32>,
    ) -> Vec<&JdNumber> {
        let mut to_return: Vec<&JdNumber> = Vec::new();
        for i in &self.id {
            if i.project != project {
                continue;
            }
            if category.is_some() {
                if category.unwrap() != i.category {
                    continue;
                }
            }
            if id.is_some() {
                if id.unwrap() != i.id {
                    continue;
                }
            }
            to_return.push(&i);
        }
        return to_return;
    }

    /// Add an id from a string.
    ///
    /// The string can be a PRO.AC number
    /// or an AC number.
    pub fn add_id_from_str(&mut self, jd: String, title: String) -> Result<(), &str> {
        let (project, category, _) = System::parse_jd_input(jd);

        if category.is_none() {
            return Err("Could not find category.");
        }

        let mut numbers = self.filter_id(project, category, None);
        numbers.sort();

        // now the last number should be highest.
        let number = numbers[numbers.len() - 1];

        let mut jd = match JdNumber::new(
            &number.area_label,
            &number.category_label,
            number.category,
            number.id + 1,
            number.project,
            number.project_label.clone(),
            number.project_area_label.clone(),
            title,
            PathBuf::new(),
        ) {
            Ok(x) => x,
            Err(_) => return Err("Could not create JD number."),
        };

        jd.path = crate::jdnumber::Location::Path(self.path.join(jd.get_relative_path()));

        self.add_id(jd)?;

        return Ok(());
    }

    /// Get an id from the system.
    pub fn get_id(&self, id: JdNumber) -> Result<JdNumber, &str> {
        match self.id.binary_search(&id) {
            Ok(index) => Ok(self.id[index].clone()),
            Err(_) => Err("Could not find JD"),
        }
    }

    /// Parse a list of paths
    pub fn from_string(_strings: Vec<String>) {
        todo!();
    }

    // /// Search for a value, using fuzzy search.
    // pub fn search(&self, input: String) {
    //     //let list: Vec<String> = self.id.clone().into_iter().map(|s| s.to_string()).collect();
    //     //let result = fuzzy_search_sorted(&input, list.into_iter().map(|s| s.as_str()).collect());

    //     let mut list: Vec<&str> = Vec::new();
    //     for id in self.id.clone() {
    //         list.push(id.to_string().as_str());
    //     }
    // }

    /// Match a PRO project number.
    fn match_project(input: &str) -> IResult<&str, u32> {
        // count(is_digit(take(1)), 3);
        recognize(count(terminated(one_of("0123456789"), many0(char('<'))), 3))(input)
            .map(|(next_input, res)| (next_input, res.parse().unwrap()))
    }

    /// Match an AC area number.
    /// ```
    /// assert_eq!(match_area("53"),Ok(("","53")));
    /// ```
    fn match_area(input: &str) -> IResult<&str, u32> {
        recognize(count(terminated(one_of("0123456789"), many0(char('<'))), 2))(input)
            .map(|(n_i, res)| (n_i, res.parse().expect("Parser forces numbers")))
    }

    /// Match an ID number.
    /// ```
    /// assert_eq!(match_id("43"),Ok(("","43")));
    /// ```
    fn match_id(input: &str) -> IResult<&str, u32> {
        recognize(count(terminated(one_of("0123456789"), many0(char('<'))), 2))(input)
            .map(|(n_i, res)| (n_i, res.parse().unwrap()))
    }

    /// Match a PRO-PRO range.
    ///
    /// For example,
    ///
    /// ```
    /// assert_eq!(
    ///     match_project_range("500-599"),
    ///     Ok(("",("599","500"))));
    /// ```
    fn match_project_range(input: &str) -> IResult<&str, (u32, u32)> {
        separated_pair(System::match_project, char('-'), System::match_project)(input)
    }

    /// Match a project area line:
    /// `500-599 Project area`
    fn project_area_line(i: &str) -> IResult<&str, ((u32, u32), &str, ())> {
        tuple((
            multispace0,
            System::match_project_range,
            not_line_ending,
            System::consume_newline,
        ))(i)
        .map(|x| (x.0, (x.1 .1, x.1 .2, x.1 .3)))
    }

    /// Match a project line;
    /// `501 project`
    fn project_line(i: &str) -> IResult<&str, (u32, &str, ())> {
        tuple((
            multispace0,
            System::match_project,
            not_line_ending,
            System::consume_newline,
        ))(i)
        .map(|i| (i.0, (i.1 .1, i.1 .2, i.1 .3))) // don't return the whitespace.
    }

    fn match_area_range(input: &str) -> IResult<&str, (u32, u32)> {
        separated_pair(System::match_area, char('-'), System::match_area)(input)
    }

    /// Parse an area line
    /// `10-19 This is the area name`
    fn area_line(input: &str) -> IResult<&str, ((u32, u32), &str, ())> {
        tuple((
            multispace0,
            System::match_area_range,
            not_line_ending,
            System::consume_newline,
        ))(input)
        .map(|i| (i.0, (i.1 .1, i.1 .2, i.1 .3))) // do a mapping so as not to return the space.
    }

    /// Parse a Jd line
    /// `50.42 label`
    fn jd_line(
        input: &str,
    ) -> IResult<&str, (Option<u32>, Option<char>, u32, char, u32, &str, ())> {
        tuple((
            multispace0,
            opt(System::match_project),
            opt(char('.')),
            System::match_area,
            char('.'),
            System::match_id,
            not_line_ending,
            System::consume_newline,
        ))(input)
        .map(|i| {
            (
                i.0,
                (i.1 .1, i.1 .2, i.1 .3, i.1 .4, i.1 .5, i.1 .6, i.1 .7),
            )
        })
    }

    fn consume_newline(i: &str) -> IResult<&str, ()> {
        value((), opt(line_ending))(i)
    }

    fn category_line(input: &str) -> IResult<&str, (u32, &str, ())> {
        let (unmatched, (_ws, (), area, label, ())) = match tuple((
            multispace0,
            not(System::match_area_range),
            System::match_area,
            not_line_ending,
            System::consume_newline,
        ))(input)
        {
            Ok(x) => x,
            Err(e) => return Err(e),
        };

        return Ok((unmatched, (area, label, ())));
    }

    pub fn parse(input: &str) -> Result<System, &str> {
        let with_projects = System::parse_with_projects(input);
        let without_projects = System::parse_without_projects(input);

        let system: System;

        // if the system uses projects:
        if with_projects.is_ok() && without_projects.is_err() {
            system = with_projects.expect("If statement makes it be OK");
        } else if with_projects.is_err() && without_projects.is_ok() {
            system = without_projects.expect("If statement makes it be OK");
        } else if with_projects.is_err() && without_projects.is_err() {
            return Err("Could not parse system.");
        } else {
            panic!("Both functions should not return Ok");
        }

        return Ok(system);
    }

    /// Parse a system
    fn parse_without_projects(input: &str) -> Result<System, &str> {
        let (_unparsed, areas) = match many1(tuple((
            System::area_line,
            many0(pair(System::category_line, many0(System::jd_line))),
        )))(input)
        {
            Ok(m) => m,
            Err(_e) => {
                return Err("Error parsing");
            }
        };

        let mut system = System::new(PathBuf::new());

        // iterate through the areas
        for (((first, last), area_label, _), categories) in areas {
            if first % 10 != 0 {
                return Err("Not a multiple of 10");
            }
            if first + 9 != last {
                return Err("Not a difference of 9");
            }
            for ((number, category_label, _), ids) in categories {
                if !(first <= number && number <= last) {
                    return Err("Category not between area limits");
                }

                for (project, _, ac, _, id, label, _) in ids {
                    let jd = match JdNumber::new(
                        area_label,
                        category_label,
                        ac,
                        id,
                        project,
                        None,
                        None,
                        label.to_string(),
                        PathBuf::new(),
                    ) {
                        Ok(j) => j,
                        Err(_) => return Err("Could not create a jd number"),
                    };
                    match system.add_id(jd) {
                        Ok(_) => {}
                        Err(_) => return Err("Could not add jd number"),
                    };
                }
            }
        }

        return Ok(system);
    }

    /// Parse a system with project notation
    fn parse_with_projects(input: &str) -> Result<System, &str> {
        let (_unparsed, project_areas) = match many1(pair(
            System::project_area_line,
            many0(pair(
                System::project_line,
                many0(pair(
                    System::area_line,
                    many0(pair(System::category_line, many0(System::jd_line))),
                )),
            )),
        ))(input)
        {
            Ok(m) => m,
            Err(_e) => {
                return Err("Error parsing");
            }
        };

        let mut system = System::new(PathBuf::new());

        // iterate through the areas

        for (((project_first, project_last), project_area_label, _), projects) in project_areas {
            for ((project, project_label, _), areas) in projects {
                if !(project_first <= project && project <= project_last) {
                    return Err("Project not between project ranges");
                }

                if project_first % 100 != 0 {
                    return Err("First project number not a multiple of 100");
                }

                if project_first + 99 != project_last {
                    return Err("Second project number not 99 greater than first");
                }
                for (((first, last), area_label, _), categories) in areas {
                    if first % 10 != 0 {
                        return Err("Not a multiple of 10");
                    }
                    if first + 9 != last {
                        return Err("Not a difference of 9");
                    }
                    for ((number, category_label, _), ids) in categories {
                        if !(first <= number && number <= last) {
                            return Err("Category not between area limits");
                        }

                        for (jd_project, _, ac, _, id, label, _) in ids {
                            if jd_project != Some(project) {
                                return Err("Project numbers do not match");
                            }

                            let jd = match JdNumber::new(
                                area_label,
                                category_label,
                                ac,
                                id,
                                jd_project,
                                Some(project_area_label.to_string()),
                                Some(project_label.to_string()),
                                label.to_string(),
                                PathBuf::new(),
                            ) {
                                Ok(j) => j,
                                Err(_) => return Err("Could not create a jd number"),
                            };
                            match system.add_id(jd) {
                                Ok(_) => {}
                                Err(_) => return Err("Could not add jd number"),
                            };
                        }
                    }
                }
            }
        }

        return Ok(system);
    }
}

impl std::fmt::Display for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut to_write = String::new();
        let mut area_string = String::new();
        let mut category_string = String::new();
        for i in &self.id {
            if i.area_label != area_string {
                area_string = i.area_label.clone();
                to_write.push_str(i.get_area().as_str());
                to_write.push_str("\n");
            }
            if i.category_label != category_string {
                category_string = i.category_label.clone();
                to_write.push_str(format!("  {}{}", i.category, &i.category_label).as_str());
                to_write.push_str("\n");
            }

            to_write.push_str(format!("    {}", i.to_string().as_str()).as_str());
            to_write.push_str("\n");
        }
        write!(f, "{}", to_write)
    }
}

#[cfg(test)]
mod tests {
    use colored::Colorize;

    use crate::{jdnumber::JdNumber, system::System};
    use std::path::PathBuf;

    #[test]
    fn test_duplicates() {
        let mut system = System::new(PathBuf::from("~"));
        let jd_1 = JdNumber::new(
            "area-label",
            "cat-label",
            50,
            42,
            None,
            None,
            None,
            "label".to_string(),
            PathBuf::new(),
        )
        .unwrap();
        system.add_id(jd_1.clone()).unwrap();
        assert!(system.add_id(jd_1.clone()).is_err());
        let jd_2 = JdNumber::new(
            "area-lab2",
            "cat-labe2",
            60,
            22,
            None,
            None,
            None,
            "label".to_string(),
            PathBuf::new(),
        )
        .unwrap();

        assert!(system.add_id(jd_2).is_ok());
        assert_eq!(system.id.len(), 2);
    }

    #[test]
    fn test_sorting() {
        let mut system1 = System::new(PathBuf::from("~"));
        let mut system2 = System::new(PathBuf::from("~"));
        // create jd numbers
        let jd_1 = JdNumber::new(
            "area-label",
            "cat-label",
            50,
            42,
            None,
            None,
            None,
            "label".to_string(),
            PathBuf::new(),
        )
        .unwrap();
        let jd_2 = JdNumber::new(
            "area-lab2",
            "cat-lab2",
            60,
            22,
            None,
            None,
            None,
            "label".to_string(),
            PathBuf::new(),
        )
        .unwrap();
        let jd_3 = JdNumber::new(
            "ar",
            "cat",
            50,
            21,
            None,
            None,
            None,
            "label_".to_string(),
            PathBuf::new(),
        )
        .unwrap();
        let jd_4 = JdNumber::new(
            "ra",
            "tac",
            10,
            5,
            None,
            None,
            None,
            "aleb".to_string(),
            PathBuf::new(),
        )
        .unwrap();

        // add jd numbers to the system in 1 order
        system1.add_id(jd_1.clone()).unwrap();
        system1.add_id(jd_2.clone()).unwrap();
        system1.add_id(jd_3.clone()).unwrap();
        system1.add_id(jd_4.clone()).unwrap();

        // add them again in a different order
        system2.add_id(jd_3).unwrap();
        system2.add_id(jd_2).unwrap();
        system2.add_id(jd_1).unwrap();
        system2.add_id(jd_4).unwrap();

        // assert that the orders are the same.
        assert_eq!(system1.id, system2.id);

        let mut system1_sorted = system1.id.clone();
        system1_sorted.sort();
        // assert that they are sorted.
        assert_eq!(system1.id, system1_sorted);
    }

    /// Create a test system
    fn create_sample_system() -> System {
        let text = r#"
(path:"/home/calvin/200-299_programming/johnnydecimal/jd",
projects:[],
id:[(project:None,category:12,id:1,label:"_sept_payroll",area_label:"_finance",category_label:"_payroll",path:Path("jd/10-19_finance/12_payroll/12.01_sept_payroll")),
	(project:None,category:12,id:2,label:"_oct_payroll",area_label:"_finance",category_label:"_payroll",path:Path("jd/10-19_finance/12_payroll/12.02_oct_payroll")),
	(project:None,category:22,id:1,label:"_cleaning_contract",area_label:"_admin",category_label:"_contracts",path:Path("jd/20-29_admin/22_contracts/22.01_cleaning_contract")),
	(project:None,category:22,id:2,label:"_office_lease",area_label:"_admin",category_label:"_contracts",path:Path("jd/20-29_admin/22_contracts/22.02_office_lease"))
])
"#;
        let system: System = ron::from_str(text).expect("Hardcoded value is valid.");
        return system;
    }
    #[test]
    fn test_show() {
        let system = create_sample_system();

        // test giving no argument
        let mut left = system.display(None).unwrap();
        let full_system = "  10-19_finance
    12_payroll
      12.01_sept_payroll
      12.02_oct_payroll
  20-29_admin
    22_contracts
      22.01_cleaning_contract
      22.02_office_lease\n";

        assert_eq!(left, full_system);

        // test giving a category
        let category = "  10-19_finance
    12_payroll
      12.01_sept_payroll
      12.02_oct_payroll\n";
        left = system.display(Some(String::from("12"))).unwrap();
        assert_eq!(left, category);

        // test giving a complete AC.ID number
        let jd_number = "  20-29_admin
    22_contracts
      22.01_cleaning_contract\n";
        left = system.display(Some(String::from("22.01"))).unwrap();
        assert_eq!(left, jd_number);

        // test giving giberish
        left = system
            .display(Some(String::from("this-is_some~giberish")))
            .unwrap();
        assert_eq!(left, full_system);

        // test giving an empty string
        left = system.display(Some(String::from(""))).unwrap();
        assert_eq!(left, full_system);
    }

    #[test]
    fn test_colorize() {
        let string1 = "Hello world.".red();
        let string2 = "Hello world.";

        assert_ne!(string1.to_string().as_str(), string2);
        assert_eq!(string1.clear().to_string().as_str(), string2);

        let string3 = format!("{}:{}{}", "Red".red(), "Blue".blue(), "green".green());
        let string4 = "Red:Bluegreen";

        assert_ne!(string3.to_string().as_str(), string4);
        assert_ne!(
            Colorize::clear(string3.clone().as_str())
                .to_string()
                .as_str(),
            string4
        );
    }

    // DEPRECATED

    //#[test]
    // fn _test_search() {
    //     let system = create_sample_system();
    //     let result1 = JdNumber::new(
    //         "_finance",
    //         "_payroll",
    //         12,
    //         1,
    //         None,
    //         None,
    //         "_sept_payroll".to_string(),
    //         PathBuf::new(),
    //     )
    //     .unwrap();

    //     assert_eq!(system.show("12.01").unwrap(), result1);
    //     assert!(system.show("this_is_gibberish").is_err());
    //     assert_eq!(
    //         system.show("12.1").err().unwrap(),
    //         "Invalid search term.  Search term should be a valid JD number."
    //     );

    //     assert_eq!(system.show("50.02").err().unwrap(), "Cannot find number.");
    //     assert_eq!(
    //         system.show("").err().unwrap(),
    //         "Invalid search term.  Search term should be a valid JD number."
    //     );
    // }
    #[test]
    fn test_get_id() {
        let system = create_sample_system();

        let jd1 = JdNumber::try_from("12.01".to_string()).unwrap();
        let jd2 = JdNumber::new(
            "_finance",
            "_payroll",
            12,
            01,
            None,
            None,
            None,
            "_sept_payroll".to_string(),
            PathBuf::from("jd/10-19_finance/12_payroll/12.01_sept_payroll"),
        )
        .unwrap();

        assert!(JdNumber::check_exactly_equal(
            system.get_id(jd1).unwrap(),
            jd2
        ));

        let jd3 = JdNumber::new(
            "",
            "",
            50,
            32,
            None,
            None,
            None,
            "l".to_string(),
            PathBuf::new(),
        )
        .unwrap();

        assert!(system.get_id(jd3).is_err());
    }

    #[test]
    fn test_add_id_from_str() {
        let mut system = create_sample_system();
        system
            .add_id_from_str("12".to_string(), "_a_title".to_string())
            .unwrap();

        assert_eq!(
            system.id[2], //because it is sorted, it is the third element.
            JdNumber::new(
                "_finance",
                "_payroll",
                12,
                03,
                None,
                None,
                None,
                "_a_title".to_string(),
                PathBuf::from("jd/10-19_finance/12_payroll/12.03_a_title")
            )
            .expect("Manual JD to be valid")
        );

        assert!(system
            .add_id_from_str("glasdf".to_string(), "s".to_string())
            .is_err());

        system
            .add_id_from_str("12".to_string(), "_a_title".to_string())
            .unwrap();

        assert_eq!(
            system.id[3],
            JdNumber::new(
                "_finance",
                "_payroll",
                12,
                04,
                None,
                None,
                None,
                "_a_title".to_string(),
                PathBuf::from("jd/10-19_finance/12_payroll/12.03_a_title")
            )
            .expect("Manual JD to be valid")
        );

        // make there be 99 ids in the category.
        for i in 0..95 {
            system
                .add_id_from_str("12".to_string(), format!("_jd_number_{}", i))
                .unwrap();
        }

        assert!(system
            .add_id_from_str("12".to_string(), "_should_fail".to_string())
            .is_err());
    }
    #[test]
    fn test_match_projects() {
        assert_eq!(System::match_project("502"), Ok(("", 502)));
        // assert_eq!(System::match_project("552432"), Ok(("", "552432")));
        assert_eq!(System::match_project("552432"), Ok(("432", 552)));
        assert!(System::match_project("project").is_err());

        assert_eq!(
            System::match_project("500-599_project_name"),
            Ok(("-599_project_name", 500))
        );
    }

    #[test]
    fn test_area_line() {
        assert_eq!(
            System::area_line("50-59_area_name\n"),
            Ok(("", ((50, 59), "_area_name", ())))
        );

        assert_eq!(
            System::area_line("10-19 area name2\n"),
            Ok(("", ((10, 19), " area name2", ())))
        );

        assert_eq!(
            System::area_line("10-19 Area\n\t11 Category"),
            Ok(("\t11 Category", ((10, 19), " Area", ())))
        );
    }

    #[test]
    fn test_jd_line() {
        assert_eq!(
            System::jd_line("50.42 Test label"),
            Ok(("", (None, None, 50, '.', 42, " Test label", ())))
        );

        assert_eq!(
            System::jd_line("104.10.53_testing"),
            Ok(("", (Some(104), Some('.'), 10, '.', 53, "_testing", ())))
        );

        assert_eq!(
            System::jd_line("10.99_hi\nThis is extra."),
            Ok(("This is extra.", (None, None, 10, '.', 99, "_hi", ())))
        );
    }

    #[test]
    fn test_category_line() {
        assert_eq!(
            System::category_line("12 Category"),
            Ok(("", (12, " Category", ())))
        );

        assert!(System::category_line("some_giberish").is_err());

        assert_eq!(
            System::category_line("50_hi\n50.01 jd label"),
            Ok(("50.01 jd label", (50, "_hi", ())))
        );
    }

    #[test]
    fn test_project_area_line() {
        assert_eq!(
            System::project_area_line("100-199_Project_1"),
            Ok(("", ((100, 199), "_Project_1", ())))
        );

        assert_eq!(
            System::project_area_line("500-599 another project\n50-59 area"),
            Ok(("50-59 area", ((500, 599), " another project", ())))
        );

        assert!(System::project_area_line("this is some gibberish").is_err());
        assert!(System::project_area_line("50-59 area").is_err());
    }

    #[test]
    fn test_project_line() {
        assert_eq!(
            System::project_line("105 Project"),
            Ok(("", (105, " Project", ())))
        );

        assert_eq!(
            System::project_line("502_project2"),
            Ok(("", (502, "_project2", ())))
        );

        assert_eq!(
            System::project_line("107 project\n50-59 area"),
            Ok(("50-59 area", (107, " project", ())))
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            System::parse(
                "100-199_project_area_name
101_project_name
10-19_finance
12_payroll
101.12.01_oct_payroll
20-29_admin
22_contracts
101.22.01_cleaning_contract
101.22.02_office_lease"
            )
            .unwrap(),
            System {
                path: PathBuf::new(),
                id: vec![
                    JdNumber::new(
                        "_finance",
                        "_payroll",
                        12,
                        01,
                        Some(101),
                        Some("_project_name".to_string()),
                        Some("_project_area_name".to_string()),
                        "_oct_payroll".to_string(),
                        PathBuf::new()
                    )
                    .unwrap(),
                    JdNumber::new(
                        "_admin",
                        "_contracts",
                        22,
                        01,
                        Some(101),
                        Some("_project_name".to_string()),
                        Some("_project_area_name".to_string()),
                        "_cleaning_contract".to_string(),
                        PathBuf::new()
                    )
                    .unwrap(),
                    JdNumber::new(
                        "_admin",
                        "_contracts",
                        22,
                        02,
                        Some(101),
                        Some("_project_name".to_string()),
                        Some("_project_area_name".to_string()),
                        "_office_lease".to_string(),
                        PathBuf::new()
                    )
                    .unwrap()
                ],
            }
        );

        assert_eq!(
            System::parse(
                "10-19_finance
    12_payroll
        12.01_oct_payroll
20-29_admin
    22_contracts
        22.01_cleaning_contract
        22.02_office_lease"
            )
            .unwrap(),
            System {
                path: PathBuf::new(),
                id: vec![
                    JdNumber::new(
                        "_finance",
                        "_payroll",
                        12,
                        01,
                        None,
                        None,
                        None,
                        "_oct_payroll".to_string(),
                        PathBuf::new()
                    )
                    .unwrap(),
                    JdNumber::new(
                        "_admin",
                        "_contracts",
                        22,
                        01,
                        None,
                        None,
                        None,
                        "_cleaning_contract".to_string(),
                        PathBuf::new()
                    )
                    .unwrap(),
                    JdNumber::new(
                        "_admin",
                        "_contracts",
                        22,
                        02,
                        None,
                        None,
                        None,
                        "_office_lease".to_string(),
                        PathBuf::new()
                    )
                    .unwrap()
                ],
            }
        );

        assert!(System::parse("this is some giberish").is_err());
    }
}
