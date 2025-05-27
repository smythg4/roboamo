use calamine::{Reader, open_workbook, Xlsx, Data};
use chrono::NaiveDate;
use regex::Regex;
use once_cell::sync::Lazy;
use std::path::Path;

use std::collections::HashMap;

use crate::things::{DutyStatus, GenericError, Person};
//use crate::csv_funcs::write_asm_to_csv;

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
    Regex::new(r"^[a-zA-z\s,]*[cC][S|M|MD]+$")
        .expect("Invalid MMCPO_REGEX pattern")
});

static AZ_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z,\s]*\sAZ+[a-zA-Z0-9]$")
        .expect("Invalid AZ_REGEX pattern")
});

fn get_qual_table(path: &Path) -> Result<HashMap<String, String>, GenericError> {
    let mut qual_table: HashMap<String, String> = HashMap::new();
    let mut reader = csv::Reader::from_path(path)?;

    for result in reader.records() {
        let record = result?;
        let asm_name = record.get(0)
            .ok_or("Missing ASM name in column 0")?
            .to_string();
        let qual_name = record.get(1)
            .ok_or("Missing qual name in column 1")?
            .to_string();
        qual_table.entry(asm_name).or_insert(qual_name);
    }
    Ok(qual_table)
}

/// Convert Data enum to a string safely
pub fn data_to_string(data: &Data) -> String {

    // using std::borrow::Cow might save me some performance in the future
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
    NAME_REGEX.is_match(text.trim())
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

fn fltmps_prd_to_date(prd_str: &str) -> Result<NaiveDate, chrono::ParseError> {
    let with_day = format!("01/{}", prd_str.trim());
    NaiveDate::parse_from_str(&with_day, "%d/%m/%Y")
}

fn parse_fltmps_file() -> Result<HashMap<String, Option<NaiveDate>>,GenericError> {
    let fltmps_path = Path::new("data/PeopleMaster.xlsx");
    println!("Attempting to open {}", fltmps_path.to_str().unwrap());
    let mut workbook: Xlsx<_> = open_workbook(fltmps_path)?;
    println!("Success opening {}", fltmps_path.to_str().unwrap());

    let mut prds: HashMap<String, Option<NaiveDate>> = HashMap::new();
    let mut name_col: usize = 0;
    let mut prd_col: usize = 0;

    if let Ok(range) = workbook.worksheet_range("FLTPMS") {
        for row in range.rows() {
            let cells = row.iter().map(data_to_string)
                                .collect::<Vec<_>>();
            if cells.iter().any(|item| item.contains("\u{a0}PRD\u{a0}")) {
                for (i, cell) in cells.iter().enumerate() {
                    if cell.contains("\u{a0}PRD\u{a0}") {
                        //println!("Found PRD at col #{}: {}", i, cell);
                        prd_col = i;
                    }
                }
                //println!("FOUND A PRD ROW!!");
                //println!("Row: {:?}", cells);
            }
            if cells.iter().any(|item| item.contains("Name")) {
                for (i, cell) in cells.iter().enumerate() {
                    if cell.contains("Name") {
                        //println!("Found Name at col #{}: {}", i, cell);
                        name_col = i;
                    }
                }
                //println!("FOUND A Name ROW!!");
                //println!("Row: {:?}", cells);
            }

            let name = cells.get(name_col)
                    .ok_or_else(|| format!("Row missing name column ({})", name_col))?
                    .clone();

            if name.is_empty() {
                continue;
            }
            let prd = match fltmps_prd_to_date(&cells[prd_col]) {
                Ok(date) => Some(date),
                Err(e) => {
                    eprintln!("Warning: Invalid PRD date for {}: {}", name, e);
                    None
                }
            };
            //println!("Name: {:?}, PRD {:?}", name, prd);
            prds.insert(name, prd);
        }
    }
    Ok(prds)
}

fn parse_asm_file() -> Result<HashMap<String, Vec<String>>,GenericError> {
    let asm_path = Path::new("data/PeopleMaster.xlsx");

    let qual_table_path = Path::new("data/qualtable.csv");
    let qual_table = get_qual_table(qual_table_path)?;

    let mut workbook: Xlsx<_> = open_workbook(asm_path)?;
    println!("Success opening {}", asm_path.to_str().unwrap());

    let mut people_quals: HashMap<String, Vec<String>> = HashMap::new();

    if let Ok(range) = workbook.worksheet_range("ASM Report") {
        let mut current_qual=String::new();
        for row in range.rows() {
            //let line = data_to_string(&row[0]);
            let line = row.get(0).map(data_to_string).unwrap_or_default();
            if is_supply(&line) && is_name(&line) {
                let person = people_quals.entry(line.clone()).or_default();
                person.push("Supply".to_string());
            }

            if let Some(qual_name) = qual_table.get(line.trim()) {
                println!("✅ Found TRACKED qual: '{line}' -> '{qual_name}'");
                current_qual = qual_name.clone();
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
    SUPPLY_REGEX.is_match(name.trim())
}

fn is_chief(name: &str) -> bool {
    CHIEF_REGEX.is_match(name.trim())
}

fn is_mmcpo(name: &str) -> bool {
    // matches any senior or master chief
    MMCPO_REGEX.is_match(name.trim())
}

fn is_az(name: &str, quals: &[String]) -> bool {
    AZ_REGEX.is_match(name.trim()) && quals.contains(&"L&R".to_string())
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
            //println!("Nothing found for: {}", last_name);
    }

    None
}

fn make_people_vec(people_quals: HashMap<String, Vec<String>>, prds: HashMap<String, Option<NaiveDate>>) -> Result<Vec<Person>, GenericError> {
    let mut people = Vec::new();

    for (name, quals) in people_quals {
        let nameparts: Vec<&str> = name.split("  ").collect();
        let name = nameparts.first()
                .ok_or("Invalid name format: missing name")?;
        let raterank = nameparts.get(1)
                .ok_or("Invalid name format: missing name")?
                .trim();
        let prd = prd_lookup(name, &prds);
        let duty_status = match prd {
            Some(_) => DutyStatus::TAR,
            None => DutyStatus::SELRES,
        };
        people.push(Person {
            name: name.to_string(),
            raterank: raterank.to_string(),
            duty_status,
            qualifications: quals,
            prd, 
        })
    }

    Ok(people)
}

pub fn make_people_complete() -> Result<Vec<Person>, GenericError> {
    let mut people_quals = parse_asm_file()?;
    add_derivative_quals(&mut people_quals);

    let prds = parse_fltmps_file()?;

    make_people_vec(people_quals, prds)
}

#[cfg(test)]
mod parser_tests {
    use super::{add_derivative_quals, make_people_vec, parse_asm_file, parse_fltmps_file};

    #[test]
    fn people_vec() {
        let mut people_quals = parse_asm_file().unwrap();
        add_derivative_quals(&mut people_quals);
        let prds = parse_fltmps_file().unwrap();

        let my_folks = make_people_vec(people_quals, prds).unwrap();
        for person in &my_folks {
            println!("Name: {}, RateRank: {}, Status: {}, PRD: {:?}, Quals: {:?}", 
            person.get_name(), 
            person.get_raterank(),
            person.get_duty_status().as_str(),
            person.get_prd(), 
            person.get_quals());
        }
        //not a great assertion...
        assert!(my_folks.len() > 0);
    }

    #[test]
    fn fltmps() {
        let prds = parse_fltmps_file().unwrap();
        println!("{:?}", prds);
    }
}

