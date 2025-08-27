use chrono::NaiveDate;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub struct Person {
    // you can privatize this later
    pub name: String,
    pub raterank: String,
    pub duty_status: DutyStatus,
    pub qualifications: Vec<String>,
    pub prd: Option<NaiveDate>,
}

impl Person {
    // pub fn get_quals(&self) -> &Vec<String> {
    //     &self.qualifications
    // }

    // pub fn get_duty_status(&self) -> &DutyStatus {
    //     &self.duty_status
    // }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    // pub fn get_raterank(&self) -> &str {
    //     &self.raterank
    // }

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
        write!(
            f,
            "({}{}): {} ({})",
            self.duty_status, prd, self.name, self.raterank
        )
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

// use crate::engine::team::{Position, Team};
// pub struct Squadron {
//     name: String,
//     people: Vec<Person>,
//     teams: Vec<Team>,
// }

// impl Person {
//     fn memory_size(&self) -> usize {
//         std::mem::size_of_val(self)
//             + self.name.capacity()
//             + self.raterank.capacity()
//             + self
//                 .qualifications
//                 .iter()
//                 .map(|q| q.capacity())
//                 .sum::<usize>()
//     }
// }
// impl Team {
//     fn memory_size(&self) -> usize {
//         std::mem::size_of_val(self)
//             + self.name.capacity()
//             + self
//                 .required_positions
//                 .iter()
//                 .map(|p| p.memory_size())
//                 .sum::<usize>()
//     }
// }

// impl Position {
//     fn memory_size(&self) -> usize {
//         std::mem::size_of_val(self) + self.qualification.capacity()
//     }
// }

// impl Squadron {
//     fn memory_footprint(&self) -> usize {
//         std::mem::size_of_val(self)
//             + self.name.capacity()
//             + self.people.iter().map(|p| p.memory_size()).sum::<usize>()
//             + self.teams.iter().map(|t| t.memory_size()).sum::<usize>()
//     }
// }
// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn test_mem_size() {
//         let qualifications = vec!["qual".to_string(); 5];
//         let person = Person {
//             name: "dummy".to_string(),
//             raterank: "AD2".to_string(),
//             duty_status: DutyStatus::TAR,
//             qualifications,
//             prd: None,
//         };
//         let pos = Position {
//             qualification: "testqual".to_string(),
//             count: 2,
//         };
//         let team = Team {
//             name: "test team".to_string(),
//             required_positions: (0..10).map(|_| pos.clone()).collect(),
//         };
//         let squadron = Squadron {
//             name: "test squadron".to_string(),
//             people: (0..100).map(|_| person.clone()).collect(),
//             teams: (0..10).map(|_| team.clone()).collect(),
//         };
//         println!("Memory usage: {}", squadron.memory_footprint());
//         assert!(false);
//     }
// }
