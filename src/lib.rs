
pub mod things {
    use chrono::NaiveDate;
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

    #[derive(Debug, Clone)]
    pub struct Assignment {
        pub person_name: String,
        pub team_name: String,
        pub qualification: String,
        pub score: i32,
    }

    #[derive(Debug, Clone)]
    pub struct AssignmentPlan {
        pub assignments: Vec<Assignment>,
        pub unfilled_positions: Vec<(String, String)>, // (team, qual)
        pub unassigned_people: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum DutyStatus {
        TAR,
        SELRES,
    }

    impl From<&str> for DutyStatus {
        fn from(other: &str) -> Self {
            match other {
                "TAR" => DutyStatus::TAR,
                _ => DutyStatus::SELRES,
            }
        }
    }

    impl From<DutyStatus> for String {
        fn from(other: DutyStatus) -> Self {
            match other {
                DutyStatus::SELRES => "SELRES".to_string(),
                DutyStatus::TAR => "TAR".to_string()
            }
        }
    }

    // impl Into<String> for DutyStatus {
    //     fn into(self) -> String {
    //         match self {
    //             DutyStatus::SELRES => "SELRES".to_string(),
    //             DutyStatus::TAR => "TAR".to_string()
    //         }
    //     }
    // }

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
}

pub mod asm_parser;
pub mod database;
pub mod csv_funcs;