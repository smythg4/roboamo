    use chrono::NaiveDate;
    use std::fmt::{Display, Formatter};
    
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