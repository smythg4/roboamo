use calamine::{Reader, open_workbook, Xlsx, Data};
use regex::Regex;
use std::path::Path;

use std::collections::HashMap;
use std::time::Instant;

use crate::things::{DutyStatus, Person};
use crate::csv_funcs::write_asm_to_csv;

fn get_qual_table(path: &Path) -> HashMap<String, String> {
    let mut qual_table: HashMap<String, String> = HashMap::new();
    let reader = csv::Reader::from_path(path);

    for result in reader.unwrap().records() {
        let record = result.unwrap();
        let asm_name = record.get(0).unwrap().to_string();
        let qual_name = record.get(1).unwrap().to_string();

        qual_table.entry(asm_name).or_insert(qual_name);
    }
    qual_table
}

/// Convert Data enum to a string safely
pub fn data_to_string(data: &Data) -> String {
    match data {
        Data::String(s) => s.clone(),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::Error(e) => format!("ERROR: {e:?}"),
        Data::DateTime(dt) => format!("{dt}"),
        _ => String::new(),
    }
}

fn is_name(text: &str) -> bool {
    let name_regex = r"^[a-zA-Z]+\s*[a-zA-Z]*\s*[a-zA-Z]*,\s+[a-zA-Z]*\s+[a-zA-Z]*\s*[a-zA-Z]*\s+[A-Z0-9]+\s*$";
    //let name_regex = r"[A-Z0-9]+\s*$";

    let namer = Regex::new(name_regex).unwrap();

    namer.is_match(text.trim())
}

fn is_qualification_line(line: &str) -> bool {
    let trimmed = line.trim();
    
    // Skip obvious non-qualification lines
    if trimmed.is_empty() || trimmed == "Name" || trimmed.contains("Report:") || trimmed.len() < 15 {
        return false;
    }
    
    // Simple check: if it's long and mostly uppercase, it's probably a qualification
    let uppercase_count = trimmed.chars().filter(|c| c.is_uppercase()).count();
    let total_letters = trimmed.chars().filter(|c| c.is_alphabetic()).count();
    
    // At least 70% of letters should be uppercase for it to be a qualification
    total_letters > 5 && (uppercase_count as f32 / total_letters as f32) > 0.7 && !is_name(line)
}

fn parse_asm_file() -> Result<HashMap<String, Vec<String>>,Box<dyn std::error::Error>> {
    let asm_path = Path::new("data/PeopleMaster.xlsx");

    let qual_table_path = Path::new("data/qualtable.csv");
    let qual_table = get_qual_table(qual_table_path);

    let mut workbook: Xlsx<_> = open_workbook(asm_path)?;
    println!("Success opening {}", asm_path.to_str().unwrap());

    let mut people_quals: HashMap<String, Vec<String>> = HashMap::new();

    if let Ok(range) = workbook.worksheet_range("ASM Report") {
        let mut current_qual=String::new();
        for row in range.rows() {
            let line = data_to_string(&row[0]);
            if is_supply(&line) && is_name(&line) {
                let person = people_quals.entry(line.clone()).or_default();
                person.push("Supply".to_string());
            }

            if let Some(qual_name) = qual_table.get(line.trim()) {
                println!("✅ Found TRACKED qual: '{line}' -> '{qual_name}'");
                current_qual.clone_from(qual_name);//current_qual = qual_name.clone();
            } else if is_name(&line) {
                if !current_qual.is_empty() {
                    let person = people_quals.entry(line.clone()).or_default();
                    if !person.contains(&current_qual) {
                        println!("👤 Found name: '{line}'. Adding qual: '{current_qual}'. Other Quals: {person:?}");
                        person.push(current_qual.clone());
                    }
                }
            } else if is_qualification_line(&line) {
                println!("❌ Found UNTRACKED qual: '{line}'");
                current_qual.clear();
            }
        }
    }
    Ok(people_quals)
}

fn is_supply(name: &str) -> bool {
    let ls_regex = r"^[a-zA-Z,\s]*\sLS+[a-zA-Z0-9]$";
    let lsre = Regex::new(ls_regex).unwrap();

    lsre.is_match(name.trim())
}

fn is_chief(name: &str) -> bool {
    // matches any Chief
    let ch_regex = r"^[a-zA-z\s,]*[cC][S|M|MD]*$";
    let chre = Regex::new(ch_regex).unwrap();

    chre.is_match(name.trim())
}

fn is_mmcpo(name: &str) -> bool {
    // matches any Senior or Master Chief
    let mch_regex = r"^[a-zA-z\s,]*[cC][S|M|MD]+$";
    let mchre = Regex::new(mch_regex).unwrap();

    mchre.is_match(name.trim())
}

fn is_az(name: &str, quals: &[String]) -> bool {
    let az_regex = r"^[a-zA-Z,\s]*\sAZ+[a-zA-Z0-9]$";
    let azre = Regex::new(az_regex).unwrap();

    azre.is_match(name.trim()) && quals.contains(&"L&R".to_string())
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

fn is_onethirty_cdi(_name: &str, quals: &[String]) -> bool {
    let all = ["13A CDI", "13B CDI"];

    all.iter().any(|item| quals.contains(&(*item).to_string()))
}

fn add_derivative_quals(people_quals: &mut HashMap<String, Vec<String>>) {

    for (name, quals) in people_quals {
        if is_az(name, quals) { quals.push("AZ".to_string()); }
        if is_chief(name) { quals.push("Chief".to_string()); }
        if is_mmcpo(name) {
            quals.push("MMCPO".to_string());
            quals.push("QAS".to_string());
            //println!("👤 Found MMCPO contender: '{name}'");
        }
        if is_fs_qar(name, quals) { quals.push("F/S QAR".to_string()); }
        if is_twohundred_cdi(name, quals) { quals.push("200 CDI".to_string()); }
        if is_onethirty_cdi(name, quals) { quals.push("130 CDI".to_string()); }
    }

}

fn make_people_vec(people_quals: HashMap<String, Vec<String>>) -> Vec<Person> {
    let mut people = Vec::new();

    for (name, quals) in people_quals {
        let nameparts: Vec<&str> = name.split("  ").collect();
        let name = *nameparts.first().unwrap();
        let raterank = *nameparts.get(1).unwrap();
        people.push(Person {
            name: name.to_string(),
            raterank: raterank.to_string(),
            duty_status: DutyStatus::TAR,
            qualifications: quals,
            prd: None, 
        })
    }
    people
}

pub fn make_people_complete() -> Vec<Person> {
    let mut people_quals = parse_asm_file().unwrap();
    add_derivative_quals(&mut people_quals);

    make_people_vec(people_quals)
}

pub fn generate_people() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let duration = start.elapsed();
    let mut people_quals = parse_asm_file()?;
    add_derivative_quals(&mut people_quals);

    let people_path = Path::new("data/people.csv");
    write_asm_to_csv(people_path, &people_quals)?;
    println!("Completion Time (ASM Parsing): {duration:?}");
    Ok(())
}

#[cfg(test)]
mod parser_tests {
    use super::{add_derivative_quals, make_people_vec, parse_asm_file};

    #[test]
    fn people_vec() {
        let mut people_quals = parse_asm_file().unwrap();
        add_derivative_quals(&mut people_quals);

        let my_folks = make_people_vec(people_quals);
        for person in &my_folks {
            println!("Name: {}, RateRank: {}, Status: {}, PRD: {:?}, Quals: {:?}", 
            person.get_name(), 
            person.get_raterank(),
            String::from(person.get_duty_status().clone()),
            person.get_prd(), 
            person.get_quals());
        }
        assert!(my_folks.len() > 0);
    }
}

