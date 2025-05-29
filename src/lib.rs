
pub mod things {
    use chrono::NaiveDate;
    use std::fmt::{Display, Formatter};

    // you can do better than this, but it works for now
    pub type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

    pub static CURRENT_DATE: NaiveDate = NaiveDate::from_ymd_opt(2025, 5, 31).unwrap();

    #[derive(Debug, Clone)]
    pub struct Team {
        pub name: String,
        pub required_positions: Vec<Position>,
    }

    #[derive(Debug, Clone)]
    pub struct Position {
        pub qualification: String,
        pub count: usize,
    }

    #[derive(Debug)]
    pub struct Assignment<'a> {
        pub person: &'a Person,
        pub team_name: String,
        pub qualification: String,
        pub optional_quals: Vec<&'a String>,
        pub score: i32,
    }

    impl<'a> Display for Assignment<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let add_quals: String = if self.optional_quals.is_empty() {
                 "".to_string()
            } else {
                format!(" - [{}]", 
                    self.optional_quals.iter().fold(String::from(""),|acc,q| acc + q))
            };
            write!(f, "<{}> {} as {} {}", &self.score, &self.person, &self.qualification, &add_quals)
        }
    }

    #[derive(Debug)]
    pub struct AssignmentPlan<'a> {
        pub assignments: Vec<Assignment<'a>>,
        pub unfilled_positions: Vec<(String, String)>, // (team, qual)
        pub unassigned_people: Vec<&'a Person>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum DutyStatus {
        TAR,
        SELRES,
    }

    impl DutyStatus {
        pub fn as_str(&self) -> &'static str {
            match self {
                DutyStatus::TAR => "TAR",
                DutyStatus::SELRES => "SELRES",
            }
        }
    }

    impl Display for DutyStatus {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.as_str())
        }
    }

    impl From<&str> for DutyStatus {
        fn from(other: &str) -> Self {
            match other {
                "TAR" => DutyStatus::TAR,
                _ => DutyStatus::SELRES,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Person {
        // you can privatize this later
        pub name: String,
        pub raterank: String,
        pub duty_status: DutyStatus,
        pub qualifications: Vec<String>,
        pub prd: Option<NaiveDate>,
    }

    impl Person {
        pub fn get_quals(&self) -> &Vec<String> {
            &self.qualifications
        }

        pub fn get_duty_status(&self) -> &DutyStatus {
            &self.duty_status
        }

        pub fn get_name(&self) -> &str {
            &self.name
        }

        pub fn get_raterank(&self) -> &str {
            &self.raterank
        }

        pub fn get_prd(&self) -> &Option<NaiveDate> {
            &self.prd
        }
    }

    impl Display for Person {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let prd = match self.prd {
                Some(date) => &format!(" - {}", date),
                None => "",
            };
            write!(f, "({}{}): {} ({})", self.duty_status, prd, self.name, self.raterank)
        }
    }
}

pub mod asm_parser;
pub mod database;
pub mod csv_funcs;