use csv::Reader;
use std::{collections::HashMap, path::Path};

type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

enum FileReadError {
    IoError,
    ParseError,
}

#[derive(Debug)]
struct Person {
    name: String,
    raterank: String,
    qualifications: Vec<String>,
}

#[derive(Debug)]
struct Team {
    name: String,
    required_positions: Vec<Position>,
}

#[derive(Debug)]
struct Position {
    qualification: String,
    count: usize,
}

fn load_teams_from_csv(path: &Path) -> Result<Vec<Team>, GenericError> {
    let mut reader = Reader::from_path(path)?;
    let mut teams_map: HashMap<String, Team> = HashMap::new();

    for result in reader.records() {
        let record = result?;
        let team_name = record.get(0).unwrap().to_string();
        let qualification = record.get(1).unwrap().to_string();
        let count: usize = record.get(2).unwrap().parse()?;

        let team = teams_map.entry(team_name.clone()).or_insert( Team {
            name: team_name,
            required_positions: Vec::new(),
        } );

        team.required_positions.push(Position { qualification, count });
    }
    Ok(teams_map.into_values().collect())
}

fn load_people_from_csv(path: &Path) -> Result<Vec<Person>, GenericError> {
    let mut reader = Reader::from_path(path)?;
    let mut people = Vec::new();

    for result in reader.records() {
      let record = result?;

      let person = Person {
        name: record.get(0).unwrap().to_string(),
        raterank: record.get(1).unwrap().to_string(),
        qualifications: record.get(2)
                        .unwrap()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect(),
      };

      people.push(person)
    }

    Ok(people)
}

// gonna need lifetime annotations on this one
fn get_qual_supply(people: &Vec<Person>) -> HashMap<String, Vec<&Person>> {
    let mut result : HashMap<String, Vec<&Person>> = HashMap::new();

    for person in people {
        for qual in &person.qualifications {
            result.entry(qual.clone()).or_default().push(person);
        }
    }
    result
}

fn get_qual_demand(teams: &Vec<Team>) -> HashMap<String, usize> {
    let mut result: HashMap<String, usize> = HashMap::new();

    for team in teams {
        for position in &team.required_positions {
            *result.entry(position.qualification.clone()).or_default() += position.count;
        }
    }
    result
}

fn main() {
    let people_path = Path::new("data/people.csv");
    let mut people = Vec::new();

    match load_people_from_csv(people_path) {
        Ok(value) => people = value,
        Err(e) => eprintln!("Error: {}", e)
    };

    let team_path = Path::new("data/teams.csv");
    let mut teams = Vec::new();

    match load_teams_from_csv(team_path) {
        Ok(value) => teams = value,
        Err(e) => eprintln!("Error: {}", e)
    };

    let qd = get_qual_demand(&teams);
    let qs = get_qual_supply(&people);
    println!("Supply: {:?}", qs);
    println!("Demand: {:?}", qd);

    let positions: Vec<(&Team, String)> = teams.iter()
        .flat_map(|t| t.required_positions.iter().map(move |q| (t.clone(), q.qualification.clone())))
        .collect();

    println!("All Positions: {:?}", positions);
}
