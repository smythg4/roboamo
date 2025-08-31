use crate::engine::{assignment::AssignmentLock, person::Person, team::{Position, Team}};
use crate::utilities::parsing::QualTable;
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete save state for a RoboAMO assignment session
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveState {
    /// Analysis date used for assignment calculations
    pub analysis_date: NaiveDate,
    
    /// All processed personnel with qualifications and PRDs
    pub people: Vec<Person>,
    
    /// All team requirements and position definitions
    pub teams: Vec<Team>,
    
    /// Qualification definitions mapping ASM codes to requirement names
    pub qual_defs: QualTable,
    
    /// Manual assignment locks 
    pub persistent_locks: Vec<AssignmentLock>,
    
    /// Timestamp when this state was exported
    pub export_timestamp: DateTime<Utc>,
    
    /// Version for compatibility tracking
    pub version: String,
}

impl SaveState {
    /// Create a new save state from current application data
    pub fn new(
        analysis_date: NaiveDate,
        people: &[Person],
        teams: &[Team],
        qual_defs: &QualTable,
        persistent_locks: &HashMap<(String, Position), String>,
    ) -> Self {
        Self {
            analysis_date,
            people: people.to_vec(),
            teams: teams.to_vec(),
            qual_defs: qual_defs.clone(),
            persistent_locks: Self::locks_to_vec(persistent_locks),
            export_timestamp: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    /// Convert Results HashMap format to export Vec format
    /// NOTE: The HashMap appears to be structured as (team_name, position) -> person_name
    /// instead of the expected (person_name, position) -> team_name
    fn locks_to_vec(locks: &HashMap<(String, Position), String>) -> Vec<AssignmentLock> {
        locks.iter().map(|((team_name, position), person_name)| {
            AssignmentLock {
                person_name: person_name.clone(),
                position: Some(position.clone()), 
                team_name: Some(team_name.clone()),
            }
        }).collect()
    }
    
    /// Convert export Vec format back to Results HashMap format
    /// NOTE: Maintaining the actual structure used: (team_name, position) -> person_name
    pub fn locks_to_hashmap(&self) -> HashMap<(String, Position), String> {
        self.persistent_locks.iter().filter_map(|lock| {
            if let (Some(position), Some(team_name)) = (&lock.position, &lock.team_name) {
                Some(((team_name.clone(), position.clone()), lock.person_name.clone()))
            } else {
                None // Skip malformed locks
            }
        }).collect()
    }
    
    /// Export save state to JSON string with pretty formatting
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .context("Failed to serialize save state to JSON")
    }
    
    /// Export save state to compact JSON string
    pub fn to_json_compact(&self) -> Result<String> {
        serde_json::to_string(self)
            .context("Failed to serialize save state to compact JSON")
    }
    
    /// Export save state as a downloadable JSON file
    #[cfg(target_arch = "wasm32")]
    pub fn download(&self, filename: &str) -> Result<()> {
        let json = self.to_json()?;
        trigger_json_download(&json, filename)
            .map_err(|e| anyhow::anyhow!("Download failed: {}", e))
    }
}

/// Trigger a browser download of JSON content
#[cfg(target_arch = "wasm32")]
fn trigger_json_download(json_content: &str, filename: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;
    
    let window = web_sys::window().ok_or("Failed to get window")?;
    let document = window.document().ok_or("Failed to get document")?;
    
    // Create a blob with the JSON content
    let array = js_sys::Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(json_content));
    
    let blob_parts = array;
    let blob_options = web_sys::BlobPropertyBag::new();
    blob_options.set_type("application/json");
    
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &blob_options)
        .map_err(|_| "Failed to create blob")?;
    
    // Create object URL
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| "Failed to create object URL")?;
    
    // Create and trigger download
    let anchor = document.create_element("a")
        .map_err(|_| "Failed to create anchor element")?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "Failed to cast to anchor element")?;
    
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();
    
    // Clean up object URL
    web_sys::Url::revoke_object_url(&url).map_err(|_| "Failed to revoke object URL")?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::person::DutyStatus;
    use std::collections::HashSet;

    #[test]
    fn test_save_state_serialization() {
        // Create test data
        let analysis_date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        
        let mut qualifications = HashSet::new();
        qualifications.insert("120 CDI".to_string());
        
        let people = vec![Person {
            name: "Smith, John".to_string(),
            raterank: "AM2".to_string(),
            duty_status: DutyStatus::Tar,
            qualifications,
            prd: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        }];
        
        let teams = vec![Team {
            name: "QA".to_string(),
            required_positions: vec![Position {
                qualification: "120 CDI".to_string(),
                instance: 1,
            }],
        }];
        
        let mut locks = HashMap::new();
        locks.insert(
            ("Smith, John".to_string(), Position { qualification: "120 CDI".to_string(), instance: 1 }),
            "QA".to_string()
        );
        
        let mut qual_defs = HashMap::new();
        qual_defs.insert("120 CDI".to_string(), "120 CDI".to_string());
        
        // Create save state and serialize
        let save_state = SaveState::new(analysis_date, &people, &teams, &qual_defs, &locks);
        let json = save_state.to_json().expect("Failed to serialize");
        
        // Verify we can deserialize back
        let deserialized: SaveState = serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(deserialized.analysis_date, analysis_date);
        assert_eq!(deserialized.people.len(), 1);
        assert_eq!(deserialized.teams.len(), 1);
        assert_eq!(deserialized.persistent_locks.len(), 1);
        assert_eq!(deserialized.version, env!("CARGO_PKG_VERSION"));
        
        // Test the conversion back to HashMap format
        let restored_locks = deserialized.locks_to_hashmap();
        assert_eq!(restored_locks.len(), 1);
        assert_eq!(restored_locks.get(&("Smith, John".to_string(), Position { qualification: "120 CDI".to_string(), instance: 1 })), Some(&"QA".to_string()));
    }
}