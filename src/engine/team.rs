use crate::engine::assignment::RoleId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub required_positions: Vec<Position>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Position {
    pub qualification: String,
    pub instance: u32,
}

impl Position {
    pub fn role_id(&self, team_name: &str) -> String {
        format!("{}-{}-{:03}", team_name, self.qualification, self.instance)
    }

    pub fn as_role_id(&self, team_name: &str) -> RoleId {
        RoleId {
            team: team_name.to_string(),
            qualification: self.qualification.clone(),
            instance: self.instance,
        }
    }
}
