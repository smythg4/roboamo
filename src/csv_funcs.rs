use crate::things::*;
use csv::Reader;
use std::{collections::HashMap, path::Path};
use chrono::NaiveDate;

pub fn load_teams_from_csv(path: &str) -> Result<Vec<Team>, GenericError> {
    let mut reader = Reader::from_path(path)?;
    let mut teams_map: HashMap<String, Team> = HashMap::new();

    for result in reader.records() {
        let record = result?;
        let team_name = record.get(0)
                .ok_or("Missing team name in column 0")?.to_string();
        let qualification = record.get(1)
                .ok_or("Missing qualification in column 1")?.to_string();
        let count: usize = record.get(2)
                .ok_or("Missing count in column 2")?
                .parse().map_err(|e| format!("Invalid count value: {}", e))?;

        let team = teams_map.entry(team_name.clone()).or_insert( Team {
            name: team_name,
            required_positions: Vec::new(),
        } );

        team.required_positions.push(Position { qualification, count });
    }
    Ok(teams_map.into_values().collect())
}

pub fn load_people_from_csv(path: &str) -> Result<Vec<Person>, GenericError> {
    let mut reader = Reader::from_path(path)?;
    let mut people = Vec::new();

    for result in reader.records() {
      let record = result?;

      let person = Person {
        name: record.get(0).unwrap().to_string(),
        raterank: record.get(1).unwrap().to_string(),
        qualifications: record.get(4)
                        .unwrap()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect(),
        duty_status: match record.get(2).unwrap_or("SELRES") {
            "TAR" | "FTS" => DutyStatus::TAR,
            _ => DutyStatus::SELRES,
        },
        prd: record.get(3)
            .and_then(|date_str| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()),
      };

      people.push(person);
    }

    Ok(people)
}

pub fn save_assignments_to_csv(assignments: &AssignmentPlan, path: &Path) -> Result<(),GenericError> {
    use csv::Writer;

    let mut writer = Writer::from_path(path)?;

    writer.write_record(["Assigned People", "", ""])?;
    writer.write_record(["Person", "Team", "Qualification"])?;

    for assignment in &assignments.assignments {
        writer.write_record([
            assignment.person.get_name(),
            &assignment.team_name,
            &assignment.qualification,
        ])?;
    }

    writer.write_record(["Vacant Positions", "", ""])?;
    writer.write_record(["Person", "Team", "Qualification"])?;
    for (team, qual) in &assignments.unfilled_positions {
        writer.write_record([
            "",
            team,
            qual,
        ])?;
    }

    writer.write_record(["Unassigned People", "", ""])?;
    writer.write_record(["Person", "Team", "Qualification"])?;
    for person in &assignments.unassigned_people {
        writer.write_record([
            person.get_name(),
            "",
            "",
        ])?;
    }

    writer.flush()?;

    Ok(())
}

pub fn write_asm_to_csv(path: &Path, people_quals: &HashMap<String, Vec<String>>) -> Result<(), std::io::Error> {
    use csv::Writer;

    let mut writer = Writer::from_path(path)?;
    writer.write_record(["Name", "RateRank", "Status", "PRD", "QualificationsList"])?;
    //Name	RateRank	Status	PRD	QualificationsList

    for (name, quals) in people_quals {
        let nameparts: Vec<&str> = name.split("  ").collect();
        let name = *nameparts.first().unwrap();
        let raterank = *nameparts.get(1).unwrap();
        let qual_str = quals.join(", ");

        writer.write_record([
            name,
            raterank,
            "TAR",
            "",
            &qual_str,
        ])?;
    }

    writer.flush()?;

    Ok(())
}