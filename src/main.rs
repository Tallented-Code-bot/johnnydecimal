use walkdir::{WalkDir,DirEntry};
use regex::Regex;


fn main() {
    index();
}



/// Create an index for a johnnydecimal system
fn index(){
    let re= Regex::new(r"\d{2}\.\d{2}.*$").unwrap();

    let walker=WalkDir::new("/home/gitpod/jd/").into_iter();
    for entry in walker.filter_entry(|e|!is_hidden(e)){
        if re.is_match(entry.as_ref().unwrap().path().to_str().unwrap()){
            //println!("match: {}",entry.unwrap().path().display());
            println!("{}",re.find(entry.unwrap().path().to_str().unwrap()).unwrap().as_str())
        }
        //println!("{}",entry.unwrap().path().display());
    }
}



/// Checks if a given file or directory is hidden.
/// 
/// Taken from https://docs.rs/walkdir/latest/walkdir/
fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}



#[derive(Debug)]
struct System{
    root:String,
    projects:Vec<String>,
    //area:Vec<String>,
    //category:Vec<String>,
    /// Looks like 100.24.34
    id:Vec<JdNumber>,
    title:Vec<String>,
}

impl System{
    /// Add an id to the system
    fn add_id(&mut self,id:JdNumber,name:String){
        self.id.push(id);
        self.title.push(name);
    }
}


/// A Johnny.Decimal number.
/// 
/// Can be either `PRO.AC.ID` or `AC.ID`.
#[derive(PartialEq,Debug)]
struct JdNumber{
    project:Option<u32>,
    category:u32,
    id:u32,
}
impl JdNumber{
    fn new(category:u32,id:u32,project:Option<u32>)->Result<Self,()>{
        // If the area or category are too long, return none
        if category>99 || id>99{
            return Err(());
        } 

        match project{
            Some(project)=>{
                if project>999{
                    return Err(());
                }
            },
            None=>{}
        } 

        return Ok(JdNumber{
            category,
            id,
            project
        })
    }
}

impl TryFrom<String> for JdNumber{
    type Error=();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        //check that there are periods in the number.
        if !value.contains("."){
            return Err(());
        }


        let numbers=value.split(".").into_iter();
        let mut new_numbers:Vec<u32>=Vec::new();

        // for each string in the generated list, parse it into
        // a number.  If it does not parse, error.
        for number in numbers{
            match number.parse(){
                Ok(x)=>{new_numbers.push(x)},
                Err(_error)=>{return Err(())}
            };
        }

        if new_numbers.len()==3{
            return JdNumber::new(new_numbers[1],new_numbers[2],Some(new_numbers[0]));
        }else{
            return JdNumber::new(new_numbers[0],new_numbers[1],None);
        }
    }
}

impl std::fmt::Display for JdNumber{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.project{
            Some(project)=>{write!(f,"{}.{}.{}",project,self.category,self.id)}
            None=>{write!(f,"{}.{}",self.category,self.id)}
        }
    }
}






// v----------------------- TESTS----------------v
#[cfg(test)]
mod tests{
    use crate::JdNumber;

    #[test]
    fn test_jd_creation(){
        assert!(JdNumber::new(100,524,None).is_err());
        assert!(JdNumber::new(43,23,None).is_ok());
        assert!(JdNumber::new(100,52,Some(402)).is_err());
        assert!(JdNumber::new(52,24,Some(2542)).is_err());
    }
    #[test]
    fn test_jd_from_string(){
        assert_eq!(JdNumber::try_from(String::from("20.35")).unwrap(),JdNumber{category:20,id:35,project:None});
        assert_eq!(JdNumber::try_from(String::from("50.32")).unwrap(),JdNumber{category:50,id:32,project:None});
        assert_eq!(JdNumber::try_from(String::from("423.62.21")).unwrap(),JdNumber{category:62,id:21,project:Some(423)});
        assert!(JdNumber::try_from(String::from("5032")).is_err());
        assert!(JdNumber::try_from(String::from("hi.by")).is_err());
        assert!(JdNumber::try_from(String::from("324.502")).is_err());
        assert!(JdNumber::try_from(String::from("3006.243.306")).is_err());
    }
    #[test]
    fn test_jd_display(){
        assert_eq!(JdNumber::try_from(String::from("20.35")).unwrap().to_string(),"20.35");
        assert_eq!(JdNumber::try_from(String::from("352.45.30")).unwrap().to_string(),"352.45.30");
    }
}