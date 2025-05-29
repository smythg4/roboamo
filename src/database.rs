use chrono::NaiveDate;
use rusqlite::{Connection, Result};
use crate::things::*;
use std::fs;

pub fn reset_db(path: &str) -> Result<(), GenericError> {
    fs::remove_file(path)?;
    Ok(())
}

pub fn create_people_table(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS person (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            raterank TEXT,
            duty_status TEXT,
            qualifications TEXT,
            prd DATE
        )",
        (), // empty list of parameters
    )?;

    Ok(conn)
}

pub fn insert_person_to_db(conn: &Connection, person: &Person) -> Result<()> {
    let quals = person.get_quals().join(",");
    let duty_status = person.get_duty_status().as_str();
    conn.execute(
        "INSERT INTO person (name, raterank, duty_status, qualifications, prd)
            VALUES (?1, ?2, ?3, ?4, ?5)",
        (person.get_name(), person.get_raterank(), duty_status, quals, person.get_prd()),
    )?;
    Ok(())
}

pub fn insert_people_to_db(conn: &mut Connection, people: &Vec<Person>) -> Result<()> {
    let tx = conn.transaction()?;
    let mut stmt = tx.prepare(
        "INSERT INTO person (name, raterank, duty_status, qualifications, prd)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    )?;
    println!("Populating members into database...");
    for person in people {
        let quals = person.get_quals().join(",");
        let duty_status = person.get_duty_status().as_str();
        stmt.execute((person.get_name(), person.get_raterank(), duty_status, quals, person.get_prd()))?;
    }
    stmt.finalize()?;
    tx.commit()?;
        println!("Database population complete.");
    Ok(())
}

pub fn fetch_people(conn: &Connection) -> Result<Vec<Person>> {
    let mut result = Vec::new();
    let mut stmt = conn.prepare("SELECT * FROM person")?;
    println!("Importing members from database...");
    let person_iter = stmt.query_map([], |row| {
        let duty_status_str: String = row.get(3)?;
        let duty_status = DutyStatus::from(&duty_status_str[..]);
        let qual_str: String = row.get(4)?;
        let qualifications: Vec<String> = qual_str.split(',').map(|s| s.to_string()).collect();
        Ok(Person {
            name: row.get(1)?,
            raterank: row.get(2)?,
            duty_status,
            qualifications,
            prd: row.get(5)?,
        })
    })?;
    for person in person_iter.flatten() {
        if person.prd.unwrap_or(NaiveDate::from_ymd_opt(2030, 12, 31).unwrap()) >= CURRENT_DATE {
            result.push(person);
        }
    }
    println!("Member import complete. {} members imported.", result.len());
    Ok(result)
}

#[cfg(test)]
mod test {
    use crate::asm_parser;

    use super::*;

    #[test]
    //#[should_panic]
    fn basics() {
        use chrono::NaiveDate;

        let path = "test.db";
        reset_db(path).unwrap();
        let mut conn = create_people_table(path).unwrap();
        let person = Person {
            name: "Smith, Stephen".to_string(),
            raterank: "LCDR".to_string(),
            duty_status: DutyStatus::TAR,
            qualifications: vec!["TAC".to_string(), "IOE IP".to_string(), "T2P".to_string()],
            prd: NaiveDate::from_ymd_opt(2026, 12, 31)
        };
        insert_person_to_db(&mut conn, &person).unwrap();
        if let Ok(peoples) = fetch_people(&conn) {
            println!("{:?}", peoples);
        }
    }

    #[test]
    fn asm_parse_to_db() {
        let path = "test.db";
        reset_db(path).unwrap();
        let mut conn = create_people_table(path).unwrap();
        let people = asm_parser::make_people_complete("data/qualifications/PeopleMaster.xlsx", "data/qual defs/qualtable.csv").unwrap();
        insert_people_to_db(&mut conn, &people).unwrap();

        match fetch_people(&conn) {
            Ok(peeps) => println!("{:?}", peeps),
            Err(e) => eprintln!("Error - {}", e)
        }
    }
}