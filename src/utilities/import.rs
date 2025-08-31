use crate::utilities::SaveState;
use anyhow::{Context, Result};

/// Import and validate a SaveState from JSON content
pub fn import_save_state(json_content: &str) -> Result<SaveState> {
    // Parse JSON
    let save_state: SaveState = serde_json::from_str(json_content)
        .context("Failed to parse JSON - file may be corrupted or invalid format")?;
    
    // Validate version compatibility
    validate_version(&save_state.version)?;
    
    // Validate data integrity
    validate_save_state(&save_state)?;
    
    Ok(save_state)
}

/// Check version compatibility
fn validate_version(version: &str) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    
    // For now, only exact version match (we can make this more flexible later)
    if version != current_version {
        return Err(anyhow::anyhow!(
            "Version mismatch: Save state created with version {} but current version is {}. Import may not work correctly.",
            version,
            current_version
        ));
    }
    
    Ok(())
}

/// Validate the imported save state data
fn validate_save_state(save_state: &SaveState) -> Result<()> {
    // Basic data validation
    if save_state.people.is_empty() {
        return Err(anyhow::anyhow!("Invalid save state: No people data found"));
    }
    
    if save_state.teams.is_empty() {
        return Err(anyhow::anyhow!("Invalid save state: No teams data found"));
    }
    
    // Validate locks reference valid people and positions
    for lock in &save_state.persistent_locks {
        // Check if the locked person exists in people data
        let person_exists = save_state.people.iter().any(|p| p.name == lock.person_name);
        if !person_exists {
            return Err(anyhow::anyhow!(
                "Invalid save state: Lock references person '{}' who doesn't exist in people data",
                lock.person_name
            ));
        }
        
        // If lock has a team_name, verify it exists
        if let Some(ref team_name) = lock.team_name {
            let team_exists = save_state.teams.iter().any(|t| t.name == *team_name);
            if !team_exists {
                return Err(anyhow::anyhow!(
                    "Invalid save state: Lock references team '{}' which doesn't exist in teams data",
                    team_name
                ));
            }
        }
        
        // If lock has a position, verify the team has that position
        if let (Some(ref team_name), Some(ref position)) = (&lock.team_name, &lock.position) {
            let team = save_state.teams.iter().find(|t| t.name == *team_name);
            if let Some(team) = team {
                let position_exists = team.required_positions.iter().any(|p| p == position);
                if !position_exists {
                    return Err(anyhow::anyhow!(
                        "Invalid save state: Lock references position '{} instance {}' which doesn't exist in team '{}'",
                        position.qualification,
                        position.instance,
                        team_name
                    ));
                }
            }
        }
    }
    
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_valid_save_state() {
        // Create a valid SaveState JSON
        let json = r#"{
            "analysis_date": "2025-01-15",
            "people": [
                {
                    "name": "Smith, John",
                    "raterank": "AM2",
                    "duty_status": "Tar",
                    "qualifications": ["120 CDI"],
                    "prd": "2025-12-31"
                }
            ],
            "teams": [
                {
                    "name": "QA",
                    "required_positions": [
                        {
                            "qualification": "120 CDI",
                            "instance": 1
                        }
                    ]
                }
            ],
            "qual_defs": {
                "120 CDI": "120 CDI"
            },
            "persistent_locks": [
                {
                    "person_name": "Smith, John",
                    "team_name": "QA",
                    "position": {
                        "qualification": "120 CDI",
                        "instance": 1
                    }
                }
            ],
            "export_timestamp": "2025-08-31T12:00:00Z",
            "version": "0.1.0"
        }"#;
        
        let result = import_save_state(json);
        assert!(result.is_ok());
        
        let save_state = result.unwrap();
        assert_eq!(save_state.people.len(), 1);
        assert_eq!(save_state.teams.len(), 1);
        assert_eq!(save_state.persistent_locks.len(), 1);
    }
    
    #[test]
    fn test_import_invalid_json() {
        let invalid_json = r#"{ "invalid": "json" }"#;
        
        let result = import_save_state(invalid_json);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_version_mismatch() {
        let json = r#"{
            "analysis_date": "2025-01-15",
            "people": [{"name": "Test", "raterank": "AM2", "duty_status": "Tar", "qualifications": [], "prd": null}],
            "teams": [{"name": "Test Team", "required_positions": []}],
            "qual_defs": {},
            "persistent_locks": [],
            "export_timestamp": "2025-08-31T12:00:00Z",
            "version": "999.0.0"
        }"#;
        
        let result = import_save_state(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Version mismatch"));
    }
}