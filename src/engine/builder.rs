use crate::engine::assignment::{AssignmentPlan, AssignmentSolver, FlowAssignment, Assignment};
use crate::utilities::*; // all the parsing logic
use crate::engine::person::{Person, DutyStatus};
use crate::engine::team::{Team, Position};
use crate::utilities::config::ParsedData;
use dioxus::prelude::*;

use std::collections::HashMap;
use regex::Regex;
use once_cell::sync::Lazy;
use chrono::NaiveDate;

use std::rc::Rc;
use anyhow::{Result, Context, bail, anyhow};

pub struct AssignmentResult {
    pub people: Rc<Vec<Person>>,
    pub teams: Rc<Vec<Team>>,
    pub flow_assignments: Vec<FlowAssignment>,
}

static NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z]+\s*[a-zA-Z]*\s*[a-zA-Z]*,\s+[a-zA-Z]*\s+[a-zA-Z]*\s*[a-zA-Z]*\s+[A-Z0-9]+\s*$")
        .expect("Invalid NAME_REGEX pattern")
});

static SUPPLY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z,\s]*\sLS+[a-zA-Z0-9]$")
        .expect("Invalid SUPPLY_REGEX pattern")
});

static CHIEF_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-z\s,]*[cC][S|M|MD]*$")
        .expect("Invalid CHIEF_REGEX pattern")
});

static MMCPO_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-z\s,]*[cC][M|MD]+$")
        .expect("Invalid MMCPO_REGEX pattern")
});

static AZ_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z,\s]*\sAZ+[a-zA-Z0-9]$")
        .expect("Invalid AZ_REGEX pattern")
});

fn get_qual_table(data: &HashMap<String, Vec<String>>) -> Result<HashMap<String, String>> {
    let mut qual_table: HashMap<String, String> = HashMap::new();

    for (common_name, asm_names) in data {
        for asm_name in asm_names {
            qual_table.insert(asm_name.to_string(), common_name.to_string());
        }
    }

    Ok(qual_table)
}

pub fn build_people() -> Result<Vec<Person>> {
    let app_state = use_context::<Signal<AppState>>();
    let files = &app_state.read().files;

    let parsed_quals = files.get("Qual Defs")
        .context("Qual Defs file not found")?
        .parsed_data.as_ref()
        .context("Qual Defs not parsed")?;

    let parsed_quals = match parsed_quals {
        ParsedData::QualDefs(quals) => quals,
        _ => bail!("Error extracting qualification definitions"),
    };

    let qual_table = get_qual_table(parsed_quals)?;

    let parsed_asm = files.get("ASM")
        .context("ASM file not found")?
        .parsed_data.as_ref()
        .context("ASM data not parsed")?;

    let parsed_asm = match parsed_asm {
        ParsedData::ASM(people) => people,
        _ => bail!("Error extracting ASM data"),
    };

    let parsed_fltmps = files.get("FLTMPS")
        .context("FLTMPS file not found")?
        .parsed_data.as_ref()
        .context("FLTMPS data not parsed")?;

    let parsed_fltmps = match parsed_fltmps {
        ParsedData::FLTMPS(prds) => prds,
        _ => bail!("Error extracting FLTMPS data"),
    };
    let mut people = vec![];

    for (name, quals) in parsed_asm.as_ref() {
        let nameparts: Vec<&str> = name.split("  ").collect();
        let name_clean = nameparts.first()
                .ok_or(anyhow!("Invalid name format: missing name"))?;
        let raterank = nameparts.get(1)
                .ok_or(anyhow!("Invalid name format: missing name"))?
                .trim();
        let prd = prd_lookup(name_clean, parsed_fltmps);
        let duty_status = match prd {
            Some(_) => DutyStatus::TAR,
            None => DutyStatus::SELRES,
        };
        let mut qualifications: Vec<String> = quals.iter()
            .filter_map(|asm_qual| qual_table.get(asm_qual))
            .map(|q| q.trim().to_uppercase().to_string() )
            .collect();

        let mut add_quals = get_derivative_quals(name, &qualifications);
        qualifications.append(&mut add_quals);

        people.push(
            Person {
                name: name_clean.to_string(),
                raterank: raterank.to_string(),
                duty_status,
                qualifications,
                prd: prd.to_owned(),
            }
        )
    }
    
    Ok(people)
}

fn is_supply(name: &str) -> bool {
    SUPPLY_REGEX.is_match(name.trim())
}

fn is_chief(name: &str) -> bool {
    CHIEF_REGEX.is_match(name.trim())
}

fn is_mmcpo(name: &str) -> bool {
    // matches any senior or master chief
    MMCPO_REGEX.is_match(name.trim())
}

fn is_az(name: &str) -> bool {
    // C-40 ASM report doesn't have a qual that matches logs and records. New plan is to just include all AZs.
    AZ_REGEX.is_match(name.trim()) //&& quals.contains(&"L&R".to_string())
}

fn is_fs_qar(_name: &str, quals: &[String]) -> bool {
    let onethirty = ["13A QAR", "13B QAR", "130 Crossrate"];
    let allothers = ["220 QAR", "210 QAR", "120 QAR", "110 QAR"];

    let has_all_others = allothers.iter().all(|item| quals.contains(&(*item).to_string()));
    let has_one_thiry = onethirty.iter().any(|item| quals.contains(&(*item).to_string()));

    has_all_others && has_one_thiry
}

fn is_twohundred_cdi(_name: &str, quals: &[String]) -> bool {
    let all = ["210 CDI", "220 CDI"];

    all.iter().all(|item| quals.contains(&(*item).to_string()))
}

fn is_onehundred_cdi(_name: &str, quals: &[String]) -> bool {
    let all = ["110 CDI", "120 CDI"];

    all.iter().all(|item| quals.contains(&(*item).to_string()))
}

fn is_onethirty_cdi(_name: &str, quals: &[String]) -> bool {
    let all = ["13A CDI", "13B CDI"];

    all.iter().any(|item| quals.contains(&(*item).to_string()))
}

fn get_derivative_quals(name: &str, quals: &[String]) -> Vec<String> {
    let mut extra_quals = vec![];
    let quals: Vec<_> = quals.iter().map(|q| q.to_uppercase()).collect();

    if is_chief(name) {
        extra_quals.push("Chief".to_string());
        extra_quals.push("QAS".to_string());
    }
    if is_mmcpo(name) {
        extra_quals.push("MMCPO".to_string());
    }
    if is_az(name) {
        extra_quals.push("AZ".to_string());
    }
    if is_supply(name) {
        extra_quals.push("Supply".to_string());
        extra_quals.push("020 SUP".to_string());
    }

    if is_fs_qar(name, &quals) { extra_quals.push("F/S QAR".to_string()); }
    if is_twohundred_cdi(name, &quals) { extra_quals.push("200 CDI".to_string()); }
    if is_onehundred_cdi(name, &quals) { extra_quals.push("100 CDI".to_string()); }
    if is_onethirty_cdi(name, &quals) { extra_quals.push("130 CDI".to_string()); }
    
    extra_quals
}

fn prd_lookup(name: &str, prds: &HashMap<String, Option<NaiveDate>>) -> Option<NaiveDate> {
    // probably will need to .to_lower() everything...

    //println!("Looking up {} in prd table...", name);
    let parts: Vec<&str> = name.splitn(2,", ").collect();
    if let [last_name, rest] = parts.as_slice() {
            //println!("Last name is {}", last_name);
            let matches: Vec<&String> = prds.keys().filter(|n| n.starts_with(last_name)).collect();
            if matches.len() == 1 {
                return prds[matches[0]];
            } else if matches.len() > 1 {
                //println!("Found multiple matches: {:?}", matches);
                let first_name = rest.split(' ').next().unwrap_or(rest);
                //println!("First name is: {}", first_name);
                let full_name = [last_name, first_name].join(" ");
                //println!("Full name would be... {}", full_name);
                let matches: Vec<&String> = prds.keys().filter(|n| n.starts_with(&full_name)).collect();
                if matches.len() == 1 {
                    return prds[matches[0]];
                }
            }
    }

    None
}

fn build_teams() -> Result<Vec<Team>> {
    let app_state = use_context::<Signal<AppState>>();
    let files = &app_state.read().files;

    let parsed_requirements = match files.get("Requirements") {
        Some(file_config) => {
            match &file_config.parsed_data {
                Some(ParsedData::Requirements(data)) => data,
                _ => panic!(),
            }
        },
        None => panic!(),
    };

    let mut teams = vec![];
    for (team_name, requirements) in parsed_requirements.as_ref() {
        teams.push( Team {
            name: team_name.clone(),
            required_positions: requirements.iter()
                .map(|r| Position{
                    qualification: r.qual_name.clone(),
                    count: r.qual_qty,
                    }).collect(),
        });
    }

    Ok(teams)
}

pub fn generate_assignments() -> Result<AssignmentResult> {
    let people = Rc::new(build_people()?);
    let teams = Rc::new(build_teams()?);
    let mut solver = AssignmentSolver::new(&people, &teams);
    let (_flow_count, _flow_cost) = solver.solve();
    let flow_assignments = solver.extract_assignments();
    Ok(AssignmentResult {
        flow_assignments,
        people: people.clone(),
        teams: teams.clone(),
    })
}

pub fn build_assignment_plan<'a>(people: &[Person], teams: &[Team], flow_assignments: &[FlowAssignment]) -> Result<AssignmentPlan, anyhow::Error> {
    let assigned_names: Vec<_> = flow_assignments.iter().map(|a| &a.person_name).collect();

    let (assigned_people, unassigned_people): (Vec<&Person>,Vec<&Person>) = people.iter()
        .partition(|p| assigned_names.contains( &&p.get_name().to_string() ));

    let mut assignments = vec![];
    for a in flow_assignments {
        let person = people.iter()
            .find(|p| p.get_name() == a.person_name)
            .ok_or_else(|| anyhow!(
                "Person {} in assignment not found in people list",
                a.person_name
            ))?;

            assignments.push( Assignment {
                person: Rc::new(person.clone()),
                team_name: a.team.clone(),
                qualification: a.qualification.clone(),
                score: 1
            });
    }

    let mut unfilled_positions = vec![];
    for team in teams {
        for position in &team.required_positions {
            let req = position.count;
            let have = flow_assignments.iter()
                .filter(|a| a.qualification == position.qualification && a.team == team.name)
                .count();
            if have < req {
                unfilled_positions.push((team.name.clone(), position.qualification.clone()))
            }
        }
    }

    Ok(AssignmentPlan{
        unassigned_people: Rc::new(unassigned_people.iter().map(|p| *p).cloned().collect()),
        assignments,
        unfilled_positions,
    })
}