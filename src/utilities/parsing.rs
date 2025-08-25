use anyhow::anyhow;
use serde::Deserialize;
use std::collections::HashMap;
use std::rc::Rc;

// everything related to qualification footprint requirements

#[derive(Debug, Clone, Deserialize)]
pub struct Requirement {
    #[serde(alias = "Name")]
    pub team_name: String,
    #[serde(alias = "Qual")]
    pub qual_name: String,
    #[serde(alias = "Num Required")]
    pub qual_qty: usize,
}

pub type Teams = HashMap<String, Vec<Requirement>>;

pub fn parse_requirements(data: Rc<Vec<u8>>) -> Result<Teams, Box<dyn std::error::Error>> {
    let mut teams = Teams::new();
    let mut rdr = csv::Reader::from_reader(&data[..]);

    for record in rdr.deserialize() {
        let record: Requirement = record?;
        teams
            .entry(record.team_name.clone())
            .or_default()
            .push(record);
    }

    Ok(teams)
}

// everything dealing with translating ASM to common qual names

pub type QualTable = HashMap<String, Vec<String>>;

pub fn parse_qual_defs(data: Rc<Vec<u8>>) -> Result<QualTable, Box<dyn std::error::Error>> {
    let mut quals = QualTable::new();
    let mut rdr = csv::Reader::from_reader(&data[..]);

    for record in rdr.records() {
        let record = record?;
        let asm_name = record
            .get(0)
            .ok_or_else(|| anyhow!("Row missing ASM name column"))?;
        let local_name = record
            .get(1)
            .ok_or_else(|| anyhow!("Row missing local name column"))?;

        quals
            .entry(local_name.to_string())
            .or_default()
            .push(asm_name.to_string());
    }

    Ok(quals)
}

// everything dealing with parsing ASM files to display members and their ASM quals
use calamine::{open_workbook_from_rs, Data, Reader, Xlsx};

use crate::engine::person::{DutyStatus, Person};

pub fn parse_asm_file(data: Rc<Vec<u8>>) -> Result<Vec<Person>, Box<dyn std::error::Error>> {
    let data = data.as_ref();
    let mut people: HashMap<String, Person> = HashMap::new();
    let cursor = std::io::Cursor::new(data);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)?;
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        for row in range.rows().skip(1) {
            if row.len() < 4 {
                //log::warn!("Skipping row with {} columns (expected 4+)", row.len());
                continue;
            }
            let qual = row
                .get(1)
                .ok_or_else(|| anyhow!("Missing qualification column at index 1"))?
                .to_string();
            let name = row
                .get(3)
                .ok_or_else(|| anyhow!("Missing name column at index 3"))?
                .to_string();
            if !name.is_empty() && !qual.is_empty() {
                let mut name_parts = name.split("  ");
                let name = name_parts.next().unwrap();
                let raterank = name_parts.last().unwrap_or("");
                let person = people.entry(name.to_string()).or_insert(Person {
                    name: name.to_string(),
                    raterank: raterank.to_string(),
                    duty_status: DutyStatus::SELRES, // this will be overridden later if needed
                    qualifications: vec![],
                    prd: None,
                });
                person.qualifications.push(qual);
            }
        }
    }
    Ok(people.into_values().collect())
}

// everything dealing with parsing FLTMPS files to display members and their PRDs
use chrono::NaiveDate;
use std::borrow::Cow;
pub type PRDList = HashMap<String, Option<NaiveDate>>;

fn fltmps_prd_to_date(prd_str: &str) -> Result<NaiveDate, chrono::ParseError> {
    let with_day = format!("01/{}", prd_str.trim());
    NaiveDate::parse_from_str(&with_day, "%d/%m/%Y")
}

fn data_to_string(data: &Data) -> Cow<'_, str> {
    // using std::borrow::Cow might save me some performance in the future
    match data {
        Data::String(s) => Cow::Borrowed(s.as_str()),
        Data::Float(f) => Cow::Owned(f.to_string()),
        Data::Int(i) => Cow::Owned(i.to_string()),
        Data::Bool(false) => Cow::Borrowed("false"),
        Data::Bool(true) => Cow::Borrowed("true"),
        Data::Error(e) => Cow::Owned(format!("ERROR: {e:?}")),
        Data::DateTime(dt) => Cow::Owned(format!("{dt}")),
        _ => Cow::Borrowed(""),
    }
}

pub fn enhance_personnel_with_prd(
    people: &mut Vec<Person>,
    prd_list: PRDList,
) -> Result<(), Box<dyn std::error::Error>> {
    for person in people {
        if let Some(prd) = prd_lookup(&person.name, &prd_list) {
            person.prd = Some(prd);
            person.duty_status = DutyStatus::TAR;
        } else {
            person.prd = None;
            person.duty_status = DutyStatus::SELRES;
        }
    }
    Ok(())
}

pub fn parse_fltmps_file(data: Rc<Vec<u8>>) -> Result<PRDList, Box<dyn std::error::Error>> {
    let data = data.as_ref();
    let mut prds = PRDList::new();
    let cursor = std::io::Cursor::new(data);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)?;

    let mut name_col: usize = 0;
    let mut prd_col: usize = 0;

    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        for row in range.rows() {
            let cells = row.iter().map(data_to_string).collect::<Vec<_>>();
            if cells.iter().any(|item| item.contains("\u{a0}PRD\u{a0}")) {
                for (i, cell) in cells.iter().enumerate() {
                    if cell.contains("\u{a0}PRD\u{a0}") {
                        prd_col = i;
                    }
                }
            }
            if cells.iter().any(|item| item.contains("Name")) {
                for (i, cell) in cells.iter().enumerate() {
                    if cell.contains("Name") {
                        name_col = i;
                    }
                }
            }

            let name = cells
                .get(name_col)
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
            prds.insert(name.to_string(), prd);
        }
    }

    Ok(prds)
}

fn prd_lookup(name: &str, prds: &HashMap<String, Option<NaiveDate>>) -> Option<NaiveDate> {
    let parts: Vec<&str> = name.splitn(2, ", ").collect();
    if let [last_name, rest] = parts.as_slice() {
        let matches: Vec<&String> = prds.keys().filter(|n| n.starts_with(last_name)).collect();
        if matches.len() == 1 {
            return prds[matches[0]];
        } else if matches.len() > 1 {
            let first_name = rest.split(' ').next().unwrap_or(rest);
            let full_name = [last_name, first_name].join(" ");
            let matches: Vec<&String> = prds.keys().filter(|n| n.starts_with(&full_name)).collect();
            if matches.len() == 1 {
                return prds[matches[0]];
            }
        }
    }

    None
}
