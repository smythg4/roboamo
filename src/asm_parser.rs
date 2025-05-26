use calamine::{Reader, open_workbook, Xlsx, Data};
use regex::Regex;
use std::path::Path;
use std::collections::HashMap;
use std::time::Instant;

fn get_qual_table(path: &Path) -> HashMap<String, String> {
    let mut qual_table: HashMap<String, String> = HashMap::new();
    if let mut reader = csv::Reader::from_path(path) {
        for result in reader.unwrap().records() {
            let record = result.unwrap();
            let asm_name = record.get(0).unwrap().to_string();
            let qual_name = record.get(1).unwrap().to_string();

            qual_table.entry(asm_name).or_insert(qual_name);
        }
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
        Data::Empty => String::new(),
        Data::Error(e) => format!("ERROR: {:?}", e),
        Data::DateTime(dt) => format!("{}", dt),
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
    total_letters > 5 && (uppercase_count as f32 / total_letters as f32) > 0.7 && !is_name(&line)
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
                let person = people_quals.entry(line.clone()).or_insert(Vec::new());
                person.push("Supply".to_string());
            }

            if let Some(qual_name) = qual_table.get(line.trim()) {
                println!("✅ Found TRACKED qual: '{}' -> '{}'", line, qual_name);
                current_qual = qual_name.clone();
            } else if is_name(&line) {
                if !current_qual.is_empty() {
                    let person = people_quals.entry(line.clone()).or_insert(Vec::new());
                    if !person.contains(&current_qual) {
                        println!("👤 Found name: '{}'. Adding qual: '{}'. Other Quals: {:?}", line, current_qual, person);
                        person.push(current_qual.clone());
                    }
                }
            } else if is_qualification_line(&line) {
                println!("❌ Found UNTRACKED qual: '{}'", line);
                current_qual.clear();
            }
        }
    }
    Ok(people_quals)
}

fn is_supply(name: &String) -> bool {
    let ls_regex = r"^[a-zA-Z,\s]*\sLS+[a-zA-Z0-9]$";
    let lsre = Regex::new(ls_regex).unwrap();

    lsre.is_match(name.trim())
}

fn is_chief(name: &String) -> bool {
    let ch_regex = r"^[a-zA-z\s,]*[cC][S|M|MD]*$";
    let chre = Regex::new(ch_regex).unwrap();

    chre.is_match(name.trim())
}

fn is_mcpo(name: &String) -> bool {
    let mch_regex = r"^[a-zA-z\s,]*[cC][M|MD]+$";
    let mchre = Regex::new(mch_regex).unwrap();

    mchre.is_match(name.trim())
}

fn is_az(name: &String, quals: &Vec<String>) -> bool {
    let az_regex = r"^[a-zA-Z,\s]*\sAZ+[a-zA-Z0-9]$";
    let azre = Regex::new(az_regex).unwrap();

    azre.is_match(name.trim()) && quals.contains(&"L&R".to_string())
}

fn is_fs_qar(name: &String, quals: &[String]) -> bool {
    let onethirty = ["13A QAR", "13B QAR", "130 Crossrate"];
    let allothers = ["220 QAR", "210 QAR", "120 QAR", "110 QAR"];

    let has_all_others = allothers.iter().all(|item| quals.contains(&item.to_string()));
    let has_one_thiry = onethirty.iter().any(|item| quals.contains(&item.to_string()));

    has_all_others && has_one_thiry
}

fn is_twohundred_cdi(name: &String, quals: &[String]) -> bool {
    let all = ["210 CDI", "220 CDI"];

    let has_all = all.iter().all(|item| quals.contains(&item.to_string()));

    has_all
}

fn is_onethirty_cdi(name: &String, quals: &[String]) -> bool {
    let all = ["13A CDI", "13B CDI"];

    let has_one = all.iter().any(|item| quals.contains(&item.to_string()));

    has_one
}

fn add_derivative_quals(people_quals: &mut HashMap<String, Vec<String>>) -> Result<(), Box<dyn std::error::Error>> {

    for (name, quals) in people_quals {
        if is_az(name, quals) { quals.push("AZ".to_string()); }
        if is_chief(name) { quals.push("Chief".to_string()); }
        if is_mcpo(name) { quals.push("MMCPO".to_string()); }
        if is_fs_qar(name, quals) { quals.push("F/S QAR".to_string()); }
        if is_twohundred_cdi(name, quals) { quals.push("200 CDI".to_string()); }
        if is_onethirty_cdi(name, quals) { quals.push("130 CDI".to_string()); }
    }
    Ok(())
}

fn write_to_csv(path: &Path, people_quals: &HashMap<String, Vec<String>>) -> Result<(), std::io::Error> {
    use csv::Writer;

    let mut writer = Writer::from_path(path)?;
    writer.write_record(&["Name", "RateRank", "Status", "PRD", "QualificationsList"])?;
    //Name	RateRank	Status	PRD	QualificationsList

    for (name, quals) in people_quals {
        let nameparts: Vec<&str> = name.split("  ").collect();
        let name = *nameparts.get(0).unwrap();
        let raterank = *nameparts.get(1).unwrap();
        let qual_str = quals.join(", ");

        writer.write_record(&[
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

pub fn generate_people() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let duration = start.elapsed();
    let mut people_quals = parse_asm_file()?;
    add_derivative_quals(&mut people_quals)?;

    let people_path = Path::new("data/people.csv");
    write_to_csv(people_path, &people_quals)?;
    println!("Completion Time (ASM Parsing): {:?}", duration);
    Ok(())
}