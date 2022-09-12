use crate::jdnumber::JdNumber;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct System {
    /// The root path of the Johnny Decimal system.
    pub path: path::PathBuf,
    pub projects: Vec<String>,
    /// A list of Johnny Decimal numbers.
    pub id: Vec<JdNumber>,
}

impl System {
    /// Add an id to the system
    pub fn add_id(&mut self, id: JdNumber) -> Result<(), &str> {
        match self.id.binary_search(&id) {
            Ok(_pos) => return Err("Element already exists."),
            Err(pos) => self.id.insert(pos, id),
        };
        return Ok(());
    }

    pub fn new(path: path::PathBuf) -> Self {
        System {
            path,
            projects: Vec::new(),
            id: Vec::new(),
        }
    }
    pub fn show(&self, search: &str) -> Result<JdNumber, &str> {
        let re = Regex::new(r"(\d{3})?\.?(\d{2})\.(\d{2})").unwrap();

        let captures = match re.captures(search) {
            Some(x) => x,
            None => return Err("Invalid search term.  Search term should be a valid JD number."),
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
            Some("project_label".to_string()),
            "label".to_string(),
            PathBuf::new(),
        )
        .unwrap();

        return match self.id.binary_search(&to_find) {
            Ok(index) => Ok(self.id[index].clone()),
            Err(_) => Err("Cannot find number."),
        };
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
    fn test_search() {
        let system = create_sample_system();
        let result1 = JdNumber::new(
            "_finance",
            "_payroll",
            12,
            1,
            None,
            None,
            "_sept_payroll".to_string(),
            PathBuf::new(),
        )
        .unwrap();

        assert_eq!(system.show("12.01").unwrap(), result1);
        assert!(system.show("this_is_gibberish").is_err());
        assert_eq!(
            system.show("12.1").err().unwrap(),
            "Invalid search term.  Search term should be a valid JD number."
        );

        assert_eq!(system.show("50.02").err().unwrap(), "Cannot find number.");
        assert_eq!(
            system.show("").err().unwrap(),
            "Invalid search term.  Search term should be a valid JD number."
        );
    }
}
