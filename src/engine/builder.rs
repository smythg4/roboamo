use crate::engine::assignment::{Assignment, AssignmentPlan, AssignmentSolver, FlowAssignment};
use crate::engine::person::Person;
use crate::engine::team::{Position, Team};
use crate::utilities::config::{AppState, ParsedData};
use dioxus::prelude::*;

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

use anyhow::{anyhow, bail, Context, Result};
use std::rc::Rc;

pub struct AssignmentResult {
    pub people: Rc<Vec<Person>>,
    pub teams: Rc<Vec<Team>>,
    pub flow_assignments: Vec<FlowAssignment>,
}

static SUPPLY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z,\s]*\sLS+[a-zA-Z0-9]$").expect("Invalid SUPPLY_REGEX pattern")
});

static CHIEF_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-z\s,]*[cC][S|M|MD]*$").expect("Invalid CHIEF_REGEX pattern"));

static MMCPO_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-z\s,]*[cC][M|MD]+$").expect("Invalid MMCPO_REGEX pattern"));

static AZ_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z,\s]*\sAZ+[a-zA-Z0-9]$").expect("Invalid AZ_REGEX pattern"));

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

    let parsed_quals = files
        .get("Qual Defs")
        .context("Qual Defs file not found")?
        .parsed_data
        .as_ref()
        .context("Qual Defs not parsed")?;

    let parsed_quals = match parsed_quals {
        ParsedData::QualDefs(quals) => quals,
        _ => bail!("Error extracting qualification definitions"),
    };

    let qual_table = get_qual_table(parsed_quals)?;

    let parsed_asm = files
        .get("ASM")
        .context("ASM file not found")?
        .parsed_data
        .as_ref()
        .context("ASM data not parsed")?;

    let mut people = match parsed_asm {
        ParsedData::Personnel(people) => people.clone(),
        _ => bail!("Error extracting ASM data"),
    };

    let people = Rc::make_mut(&mut people);

    for person in people.iter_mut() {
        person.qualifications = person
            .qualifications
            .iter()
            .filter_map(|q| qual_table.get(q))
            .cloned()
            .collect();

        let temp_name = format!("{}  {}", &person.name, &person.raterank);
        let derivative_quals = get_derivative_quals(&temp_name, &person.qualifications);
        person.qualifications.extend(derivative_quals);
    }

    Ok(people.clone())
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

    let has_all_others = allothers
        .iter()
        .all(|item| quals.contains(&(*item).to_string()));
    let has_one_thiry = onethirty
        .iter()
        .any(|item| quals.contains(&(*item).to_string()));

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

    if is_fs_qar(name, &quals) {
        extra_quals.push("F/S QAR".to_string());
    }
    if is_twohundred_cdi(name, &quals) {
        extra_quals.push("200 CDI".to_string());
    }
    if is_onehundred_cdi(name, &quals) {
        extra_quals.push("100 CDI".to_string());
    }
    if is_onethirty_cdi(name, &quals) {
        extra_quals.push("130 CDI".to_string());
    }

    extra_quals
}

fn build_teams() -> Result<Vec<Team>> {
    let app_state = use_context::<Signal<AppState>>();
    let files = &app_state.read().files;

    let parsed_requirements = files
        .get("Requirements")
        .context("Requirements file not found")?
        .parsed_data
        .as_ref()
        .context("Requirements file not parsed")?;

    let mut teams = match parsed_requirements {
        ParsedData::Requirements(teams) => teams.clone(),
        _ => bail!("Error extracting requirements data"),
    };

    let teams = Rc::make_mut(&mut teams);

    Ok(teams.clone())
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

pub fn build_assignment_plan(
    people: &[Person],
    teams: &[Team],
    flow_assignments: &[FlowAssignment],
) -> Result<AssignmentPlan, anyhow::Error> {
    let assigned_names: Vec<_> = flow_assignments.iter().map(|a| &a.person_name).collect();

    let (_assigned_people, unassigned_people): (Vec<&Person>, Vec<&Person>) = people
        .iter()
        .partition(|p| assigned_names.contains(&&p.get_name().to_string()));

    let mut assignments = vec![];
    for a in flow_assignments {
        let person = people
            .iter()
            .find(|p| p.get_name() == a.person_name)
            .ok_or_else(|| {
                anyhow!(
                    "Person {} in assignment not found in people list",
                    a.person_name
                )
            })?;

        assignments.push(Assignment {
            person: Rc::new(person.clone()),
            team_name: a.team.clone(),
            qualification: a.qualification.clone(),
            score: 1,
        });
    }

    let mut unfilled_positions = vec![];
    for team in teams {
        for position in &team.required_positions {
            let req = position.count;
            let have = flow_assignments
                .iter()
                .filter(|a| a.qualification == position.qualification && a.team == team.name)
                .count();
            if have < req {
                for _ in 0..(req - have) {
                    unfilled_positions.push((team.name.clone(), position.qualification.clone()))
                }
            }
        }
    }

    Ok(AssignmentPlan {
        unassigned_people: Rc::new(unassigned_people.iter().map(|p| *p).cloned().collect()),
        assignments,
        unfilled_positions,
    })
}
