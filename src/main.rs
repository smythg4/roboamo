use csv::Reader;
use std::{collections::{HashMap,HashSet, VecDeque}, path::Path};
use std::time::Instant;
//use rayon::prelude::*;
use chrono::{NaiveDate, Utc};
use colorize::AnsiColor;

mod asm_parser;

// you can do better than this, but it works for now
type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

static CURRENT_DATE: NaiveDate = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();

#[derive(Debug, Clone, PartialEq)]
enum DutyStatus {
    TAR,
    SELRES,
}

#[derive(Debug, Clone)]
struct Person {
    name: String,
    raterank: String,
    duty_status: DutyStatus,
    qualifications: Vec<String>,
    prd: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
struct Team {
    name: String,
    required_positions: Vec<Position>,
}

#[derive(Debug, Clone)]
struct Position {
    qualification: String,
    count: usize,
}

#[derive(Debug, Clone)]
struct Assignment {
    person_name: String,
    team_name: String,
    qualification: String,
    score: i32,
}

#[derive(Debug, Clone)]
struct AssignmentPlan {
    assignments: Vec<Assignment>,
    unfilled_positions: Vec<(String, String)>, // (team, qual)
    unassigned_people: Vec<String>,
}

fn create_positions_to_fill(teams: &[Team]) -> Vec<(Team, String)> {
    let mut positions = Vec::new();

    for team in teams {
        for position in &team.required_positions {
            for _ in 0..position.count {
                positions.push((team.clone(), position.qualification.clone()));
            }
        }
    }
    positions
}

fn calc_person_score(
    person: &Person,
    current_qual: &str,
    high_demand_quals: &HashSet<String>,
    remaining_positions: &VecDeque<(Team, String)>,
    all_people: &[Person],
    current_date: NaiveDate,
) -> i32 {
    let mut score = 0;

    if person.duty_status == DutyStatus::SELRES {
        score += 10_000;
    }

    if person.raterank.contains("AWF") {
        score += 10_000;
    }

    if let Some(prd) = person.prd {
        let days_remaining = (prd - current_date).num_days();

        if days_remaining < 90 {
            score += 5_000;
        } else if days_remaining < 180 {
            score += 2_000;
        } else if days_remaining < 365 {
            score += 500;
        }
    } else {
        // no prd known, best to avoid if possible
        score += 10_000;
    }

    score += person.qualifications.len() as i32 * 100;

    for qual in &person.qualifications {
        if qual != current_qual {
            let people_with_qual = all_people.iter()
                .filter(|p| p.qualifications.contains(qual))
                .count();

            let positions_needing_qual = remaining_positions.iter()
                .filter(|(_,q)| q == qual)
                .count();

            if positions_needing_qual > 0 && people_with_qual <= positions_needing_qual + 1 {
                score += 2_000;
            }
        }
    }

    if !high_demand_quals.contains(current_qual) {
        if person.qualifications.iter().any(|q| high_demand_quals.contains(q)) {
            score += 100;
        }
    }
    score
}

fn find_best_available_person<'a>(
    qual: &str,
    qual_supply: &HashMap<String, Vec<&'a Person>>,
    assigned_people: &HashSet<String>,
    all_people: &'a [Person],
    high_demand_quals: &HashSet<String>,
    remaining_positions: &VecDeque<(Team, String)>
) -> Option<(&'a Person, i32)> {
    if let Some(candidates) = qual_supply.get(qual) {
        let mut available_candidates: Vec<_> = candidates.iter()
            .filter(|p| !assigned_people.contains(&p.name))
            .copied()
            .collect();

        let mut scored_candidates: Vec<(&Person, i32)> = available_candidates.iter()
            .map(|person| {
                let score = calc_person_score(
                    person,
                    qual,
                    high_demand_quals,
                    remaining_positions,
                    all_people,
                    CURRENT_DATE);
                    (*person, score)
            }).collect();

            scored_candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        scored_candidates.first().map(|(person, score)| (*person, *score))
    } else {
        None
    }
}

fn assign_teams(
    people: &[Person],
    teams: &[Team],
    high_demand_quals: &HashSet<String>
) -> AssignmentPlan {
    let qual_supply = get_qual_supply(people);
    let qual_demand = get_qual_demand(teams);

    let mut qual_criticality: Vec<(String, f64)> = qual_demand.iter()
        .map(|(qual, demand)| {
            let supply = qual_supply.get(qual).map_or(0, |v| v.len());
            let ratio = supply as f64 / *demand as f64;
            (qual.clone(), ratio)
        }).collect();

    qual_criticality.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

    let mut positions_queue = VecDeque::new();

    for (qual, _ratio) in &qual_criticality {
        for team in teams {
            for position in &team.required_positions {
                if &position.qualification == qual {
                    for _ in 0..position.count {
                        positions_queue.push_back((team.clone(), qual.clone()));
                    }
                }
            }
        }
    }

    let mut assignments = Vec::new();
    let mut assigned_people = HashSet::new();
    let mut unfilled_positions = Vec::new();

    while let Some((team, qual)) = positions_queue.pop_front() {
        if let Some((best_person, score)) = find_best_available_person(
            &qual,
            &qual_supply,
            &assigned_people,
            people,
            high_demand_quals,
            &positions_queue,
        ) {
            assigned_people.insert(best_person.name.clone());
            assignments.push(Assignment {
                person_name: best_person.name.clone(),
                team_name: team.name.clone(),
                qualification: qual.clone(),
                score,
            });
        } else {
            unfilled_positions.push((team.name.clone(), qual.clone()));
        }
    }

    for (team_name, qual) in &unfilled_positions {
        for person in people {
            if !assigned_people.contains(&person.name) && person.qualifications.contains(qual) {
                assigned_people.insert(person.name.clone());
                assignments.push( Assignment {
                    person_name: person.name.clone(),
                    team_name: team_name.clone(),
                    qualification: qual.clone(),
                    score: 0,
                });
                break;
            }
        }
    }

    let unassigned_people: Vec<String> = people.iter()
        .filter(|p| !assigned_people.contains(&p.name))
        .map(|p| format!("{} ({}) - {:?}", p.name, p.raterank, p.qualifications))
        .collect();

    AssignmentPlan {
        unfilled_positions: unfilled_positions.into_iter()
            .filter(|(team, qual)| {
                !assignments.iter().any(|a| &a.team_name == team && &a.qualification == qual)
            }).collect(),
        unassigned_people,
        assignments,
    }
}

fn analyze_bottlenecks(people: &[Person], teams: &[Team]) {
    let supply = get_qual_supply(people);
    let demand = get_qual_demand(teams);
    
    println!("\n=== Supply/Demand Analysis ===");
    let mut bottlenecks = Vec::new();
    
    for (qual, &demand_count) in &demand {
        let supply_count = supply.get(qual).map_or(0, |v| v.len());
        let ratio = supply_count as f64 / demand_count as f64;
        bottlenecks.push((qual.clone(), supply_count, demand_count, ratio));
    }
    
    bottlenecks.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
    
    for (qual, supply, demand, ratio) in bottlenecks {
        let status = if ratio < 1.0 {
            "❌ SHORTAGE"
        } else if ratio < 1.2 {
            "⚠️  TIGHT"
        } else {
            "✅ OK"
        };
        
        println!("{:15} {:3} / {:3} = {:.2}  {}", 
            qual, supply, demand, ratio, status);
    }
}

fn value_to_colorize_8bit(value: f32, max_value: f32) -> String {
    let normalized = (value / max_value).clamp(0.0, 1.0);
    
    // Map to 8-bit color codes: green to red spectrum
    let color_code = match normalized {
        n if n <= 0.1 => 46,   // Bright green
        n if n <= 0.2 => 40,   // Green
        n if n <= 0.3 => 34,   // Light green
        n if n <= 0.4 => 28,   // Dark green
        n if n <= 0.5 => 226,  // Yellow
        n if n <= 0.6 => 220,  // Gold
        n if n <= 0.7 => 214,  // Orange
        n if n <= 0.8 => 208,  // Dark orange
        n if n <= 0.9 => 202,  // Red-orange
        _ => 196,              // Bright red
    };
    
    format!("\x1b[38;5;{}m", color_code)
}

fn print_results(plan: &AssignmentPlan, teams: &[Team]) {
    println!("\n=== Assignment Results ===");
    
    // Group by team
    let mut by_team: HashMap<String, Vec<&Assignment>> = HashMap::new();
    for assignment in &plan.assignments {
        by_team.entry(assignment.team_name.clone()).or_default().push(assignment);
    }
    
    // Print each team's status
    for team in teams {
        println!("\n{}:", team.name);
        
        let team_assignments = by_team.get(&team.name);
        let mut filled_positions: HashMap<String, usize> = HashMap::new();
        
        if let Some(assignments) = team_assignments {
            for assignment in assignments {
                let color = value_to_colorize_8bit(assignment.score as f32, 30000.0);
                println!("{}  {}({}) as {}\x1b[0m", color, assignment.person_name, assignment.score, assignment.qualification);
                //println!("  {} as {}", assignment.person_name, assignment.qualification);
                *filled_positions.entry(assignment.qualification.clone()).or_default() += 1;
            }
        }
        
        // Check for unfilled positions
        for position in &team.required_positions {
            let filled = filled_positions.get(&position.qualification).copied().unwrap_or(0);
            if filled < position.count {
                println!("  ❌ UNFILLED: {} {} position(s)", 
                    position.count - filled, position.qualification);
            }
        }
    }
    
    if !plan.unfilled_positions.is_empty() {
        println!("\n=== Critical Unfilled Positions ===");
        for (team, qual) in &plan.unfilled_positions {
            println!("  {} needs {}", team, qual);
        }
    }
    
    println!("\n=== Unassigned Personnel ({}) ===", plan.unassigned_people.len());
    for person in &plan.unassigned_people {
        println!("  {}", person);
    }
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

      people.push(person)
    }

    Ok(people)
}

fn save_assignments_to_csv(assignments: &AssignmentPlan, path: &Path) -> Result<(),GenericError> {
    use csv::Writer;

    let mut writer = Writer::from_path(path)?;

    writer.write_record(&["Assigned People", "", ""])?;
    writer.write_record(&["Person", "Team", "Qualification"])?;

    for assignment in &assignments.assignments {
        writer.write_record(&[
            &assignment.person_name,
            &assignment.team_name,
            &assignment.qualification,
        ])?;
    }

    writer.write_record(&["Vacant Positions", "", ""])?;
    writer.write_record(&["Person", "Team", "Qualification"])?;
    for (team, qual) in &assignments.unfilled_positions {
        writer.write_record(&[
            "",
            &team,
            &qual,
        ])?;
    }

    writer.write_record(&["Unassigned People", "", ""])?;
    writer.write_record(&["Person", "Team", "Qualification"])?;
    for person in &assignments.unassigned_people {
        writer.write_record(&[
            &person,
            "",
            "",
        ])?;
    }

    writer.flush()?;

    Ok(())
}

// gonna need lifetime annotations on this one
fn get_qual_supply(people: &[Person]) -> HashMap<String, Vec<&Person>> {
    let mut result : HashMap<String, Vec<&Person>> = HashMap::new();

    for person in people {
        for qual in &person.qualifications {
            result.entry(qual.clone()).or_default().push(person);
        }
    }
    result
}

fn get_qual_demand(teams: &[Team]) -> HashMap<String, usize> {
    let mut result: HashMap<String, usize> = HashMap::new();

    for team in teams {
        for position in &team.required_positions {
            *result.entry(position.qualification.clone()).or_default() += position.count;
        }
    }
    result
}

fn main() {

    // println!("Running parse test...");
    //let _ = asm_parser::generate_people().expect("Error loading ASM and FLTMPS files...");
    // println!("Parse test ran...");

    let start = Instant::now();
    let duration = start.elapsed();

    let people_path = Path::new("data/people.csv");
    let mut people = Vec::new();
    let team_path = Path::new("data/teams.csv");
    let mut teams = Vec::new();

    // parallel loading of data. how cute
    rayon::join(
        ||    match load_people_from_csv(people_path) {
        Ok(value) => people = value,
        Err(e) => {
            eprintln!("Error loading people: {}", e);
            return;
        }
    },
        ||     match load_teams_from_csv(team_path) {
        Ok(value) => teams = value,
        Err(e) => {
            eprintln!("Error loading teams: {}", e);
            return;
        }
    }
    );

    analyze_bottlenecks(&people, &teams);

    // push this out to a csv probably for configurability
    let mut high_demand_quals = HashSet::new();
    high_demand_quals.insert("MMCPO".to_string());
    high_demand_quals.insert("QAS".to_string());
    high_demand_quals.insert("SFF".to_string());
    high_demand_quals.insert("F/A QAR".to_string());

    let assignments = assign_teams(&people, &teams, &high_demand_quals);

    print_results(&assignments, &teams);

    // Save results to CSV. Gotta adjust to account for new AssignmentResults struct
    if let Err(e) = save_assignments_to_csv(&assignments, Path::new("data/assignments.csv")) {
        eprintln!("Error saving assignments: {}", e);
    }
    println!("Completion Time (Manning Allocation): {:?}", duration);
}
