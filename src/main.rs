
use std::{collections::{HashMap,HashSet, VecDeque}, path::Path};
use std::time::Instant;
use chrono::NaiveDate;
use clap::Parser;

use roboamo::{asm_parser::make_people_complete, database::fetch_people, things::*};
use roboamo::csv_funcs::*;
use roboamo::database::{create_people_table, insert_people_to_db, reset_db};

#[derive(Parser)]
#[command(name = "roboamo")]
#[command(about = "A CLI manpower optimazation tool.")]
struct Args {
    /// update the database (add ability to add filepaths)
    #[arg(short = 'u', long = "update")]
    update: bool,

    /// input files to process
    #[arg(short = 'f', long = "files", value_name = "FILE")]
    files: Vec<String>,
}

fn calc_person_score(
    person: &Person,
    current_qual: &str,
    high_demand_quals: &HashSet<String>,
    remaining_positions: &VecDeque<(String, String)>,
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

    score += i32::try_from(person.qualifications.len()).unwrap_or(0) * 100;

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

    if !high_demand_quals.contains(current_qual) && person.qualifications.iter().any(|q| high_demand_quals.contains(q)) {
            score += 100;
    }

    score
}

fn find_best_available_person<'a>(
    qual: &str,
    qual_supply: &HashMap<String, Vec<&'a Person>>,
    assigned_people: &HashSet<String>,
    all_people: &'a [Person],
    high_demand_quals: &HashSet<String>,
    remaining_positions: &VecDeque<(String, String)>
) -> Option<(&'a Person, i32)> {
    if let Some(candidates) = qual_supply.get(qual) {
        let available_candidates: Vec<_> = candidates.iter()
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

fn assign_teams<'a>(
    people: &'a [Person],
    teams: &[Team],
    high_demand_quals: &HashSet<String>
) -> AssignmentPlan<'a> {
    let qual_supply = get_qual_supply(people);
    let qual_demand = get_qual_demand(teams);

    let mut qual_criticality: Vec<(String, f64)> = qual_demand.iter()
        .map(|(qual, demand)| {
            let supply = qual_supply.get(qual).map_or(0, std::vec::Vec::len);//|v| v.len());
            let ratio = supply as f64 / *demand as f64;
            (qual.clone(), ratio)
        }).collect();

    qual_criticality.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

    let mut positions_queue: VecDeque<(String,String)> = VecDeque::new();

    for (qual, _ratio) in &qual_criticality {
        for team in teams {
            for position in &team.required_positions {
                if &position.qualification == qual {
                    for _ in 0..position.count {
                        positions_queue.push_back((team.name.clone(), qual.clone()));
                    }
                }
            }
        }
    }

    let mut assignments = Vec::new();
    let mut assigned_people = HashSet::new();
    let mut unfilled_positions = Vec::new();

    while let Some((team_name, qual)) = positions_queue.pop_front() {
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
                person: best_person,
                team_name: team_name.clone(),
                qualification: qual.clone(),
                score,
            });
        } else {
            unfilled_positions.push((team_name.clone(), qual.clone()));
        }
    }

    for (team_name, qual) in &unfilled_positions {
        for person in people {
            if !assigned_people.contains(&person.name) && person.qualifications.contains(qual) {
                assigned_people.insert(person.name.clone());
                assignments.push( Assignment {
                    person,
                    team_name: team_name.clone(),
                    qualification: qual.clone(),
                    score: 0,
                });
                break;
            }
        }
    }

    let unassigned_people: Vec<&Person> = people.iter()
        .filter(|p| !assigned_people.contains(&p.name))
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
        let supply_count = supply.get(qual).map_or(0, std::vec::Vec::len);//|v| v.len());
        let ratio = supply_count as f64 / demand_count as f64;
        bottlenecks.push((qual.clone(), supply_count, demand_count, ratio));
    }
    
    bottlenecks.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
    
    for (qual, supply, demand, ratio) in bottlenecks {
        let status = if ratio < 1.0 {
            "❌ SHORTAGE"
        } else if ratio < 1.3 {
            "⚠️  TIGHT"
        } else {
            "✅ OK"
        };
        
        println!("{qual:15} {supply:3} / {demand:3} = {ratio:.2}  {status}");
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
        n if n <= 0.95 => 202,  // Red-orange
        _ => 196,              // Bright red
    };
    
    format!("\x1b[38;5;{color_code}m")
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
                let color = value_to_colorize_8bit(assignment.score as f32, 20000.0);
                println!("{}  {} ({}) as {}\x1b[0m", color, assignment.person, assignment.score, assignment.qualification);
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
            println!("  {team} needs {qual}");
        }
    }
    
    println!("\n=== Unassigned Personnel ({}) ===", plan.unassigned_people.len());
    let (mut tar, mut selres): (Vec<&Person>,Vec<&Person>) = plan.unassigned_people.iter().partition(|p| p.duty_status == DutyStatus::TAR);
    let sort_key = |p: &&Person| std::cmp::Reverse(p.qualifications.len());
    tar.sort_by_key(sort_key);
    selres.sort_by_key(sort_key);
    for person in tar {
        let quals = person.get_quals().join(", ");
        println!("  {person} - {quals}");
    }
    for person in selres {
        let quals = person.get_quals().join(", ");
        println!("  {person} - {quals}");
    }
}


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

fn main() -> Result<(), GenericError> {
    let start = Instant::now();

    use rusqlite::Connection;
    let path = "test.db";
    let conn = Connection::open(path)
            .map_err(|e| format!("Failed to open database: '{}': {}", path, e))?;

    let args = Args::parse();

    if args.update {
        println!("Updating the database...");
        reset_db(path).unwrap();
        let mut conn = create_people_table(path).unwrap();
        let people = make_people_complete()
                .map_err(|e| format!("Failed to parse personnel data: {}", e))?;
        insert_people_to_db(&mut conn, &people).unwrap();
    }

    let people = fetch_people(&conn)
            .map_err(|e| format!("Failed to fetch people from the database: {}", e))?;

    let team_path = Path::new("data/teams.csv");
    let teams = load_teams_from_csv(team_path)
            .map_err(|e| format!("Failed to load teams from {:?}: {}", team_path, e))?;

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
        eprintln!("Error saving assignments: {e}");
    }
    let duration = start.elapsed();
    println!("Completion Time (Manning Allocation): {duration:?}");

    Ok(())
}
