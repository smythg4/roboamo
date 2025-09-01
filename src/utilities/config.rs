use crate::engine::person::Person;
use crate::engine::team::{Position, Team};
use crate::utilities::parsing::{PRDList, QualTable};
use std::collections::HashMap;
use std::rc::Rc;

pub const PAGES: [&str; 4] = ["Requirements", "Qual Defs", "ASM", "FLTMPS"];

const DESCS: [&str; 4] = [
    "This file defines the set of groups used within the system, including a listing of all qualifications associated with each group and the quantity of personnel required to hold each qualification.",
    "This lookup table provides a mapping between ASM qualification names and their equivalent descriptions used in your requirements file, allowing for easier interpretation.",
    "This export is generated from ASM (MMP) and contains a roster of squadron personnel along with their currently held qualifications as recorded in the system.",
    "This file is an export from FLTMPS and is used to reference Projected Rotation Dates (PRDs) for TAR sailors."
];

const PREVIEWS: [PreviewType; 4] = [
    PreviewType::Requirements,
    PreviewType::QualDef,
    PreviewType::Asm,
    PreviewType::Fltmps,
];
const FILE_TYPES: [&str; 4] = [".csv", ".csv", ".xlsx", ".xlsx"];
const NEXT_PAGES: [Option<&str>; 4] = [Some("Qual Defs"), Some("ASM"), Some("FLTMPS"), None];
const DEMO_PATHS: [Option<&str>; 4] = [
    Some("/roboamo/assets/demo/demoteams.csv"),
    Some("/roboamo/assets/demo/demoqualtable.csv"),
    Some("/roboamo/assets/demo/demoasm.xlsx"),
    Some("/roboamo/assets/demo/demofltmps.xlsx"),
];

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PreviewType {
    Requirements,
    QualDef,
    Asm,
    Fltmps,
}

#[derive(Debug, Clone)]
pub enum ParsedData {
    Requirements(Rc<Vec<Team>>),
    QualDefs(Rc<QualTable>),
    Fltmps(Rc<PRDList>),
    Personnel(Rc<Vec<Person>>), // Combined ASM + FLTMPS data
}

#[derive(Debug, Clone)]
pub struct FileUploadConfig {
    pub file_content: Option<Rc<Vec<u8>>>,
    pub file_name: Option<String>,
    pub page_desc: String,
    pub preview_type: PreviewType,
    pub file_types: String,
    pub next_page: Option<String>,
    pub parsed_data: Option<ParsedData>,
    pub demo_file_path: Option<&'static str>,
}

impl FileUploadConfig {
    pub fn clear_raw_data(&mut self) {
        if self.parsed_data.is_some() {
            self.file_content = None;
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub files: HashMap<String, FileUploadConfig>,
    pub persistent_locks: HashMap<(String, Position), String>,
}

impl Default for AppState {
    fn default() -> Self {
        let mut files = HashMap::new();
        for (i, page_name) in PAGES.iter().enumerate() {
            let file = FileUploadConfig {
                file_content: None,
                file_name: None,
                page_desc: DESCS[i].to_string(),
                preview_type: PREVIEWS[i].clone(),
                file_types: FILE_TYPES[i].to_string(),
                next_page: NEXT_PAGES[i].map(|page| page.to_string()),
                parsed_data: None,
                demo_file_path: DEMO_PATHS[i],
            };
            files.entry(page_name.to_string()).or_insert(file);
        }
        AppState {
            files,
            persistent_locks: HashMap::new(),
        }
    }
}

impl AppState {
    pub fn is_empty(&self) -> bool {
        self.files.iter().all(|file| file.1.parsed_data.is_none())
    }
    pub fn all_files_uploaded(&self) -> bool {
        // Check if this is a save state load scenario (has Requirements, ASM, Qual Defs but no FLTMPS)
        let has_requirements = self
            .files
            .get("Requirements")
            .map_or(false, |f| f.parsed_data.is_some());
        let has_asm = self
            .files
            .get("ASM")
            .map_or(false, |f| f.parsed_data.is_some());
        let has_qual_defs = self
            .files
            .get("Qual Defs")
            .map_or(false, |f| f.parsed_data.is_some());
        let has_fltmps = self
            .files
            .get("FLTMPS")
            .map_or(false, |f| f.parsed_data.is_some());

        // Save state scenario: Requirements + ASM + Qual Defs (FLTMPS data integrated into Person objects)
        if has_requirements && has_asm && has_qual_defs && !has_fltmps {
            return true;
        }

        // Normal file upload scenario: all files required
        self.files.values().all(|f| f.parsed_data.is_some())
    }

    pub fn upload_progress(&self) -> (usize, usize) {
        let current_count = self
            .files
            .values()
            .filter(|f| f.parsed_data.is_some())
            .count();

        // Check if this is a save state scenario (3 files loaded, FLTMPS data integrated)
        let has_requirements = self
            .files
            .get("Requirements")
            .map_or(false, |f| f.parsed_data.is_some());
        let has_asm = self
            .files
            .get("ASM")
            .map_or(false, |f| f.parsed_data.is_some());
        let has_qual_defs = self
            .files
            .get("Qual Defs")
            .map_or(false, |f| f.parsed_data.is_some());
        let has_fltmps = self
            .files
            .get("FLTMPS")
            .map_or(false, |f| f.parsed_data.is_some());

        if has_requirements && has_asm && has_qual_defs && !has_fltmps {
            // Save state scenario: show 4/4 even though only 3 files loaded
            (4, PAGES.len())
        } else {
            // Normal scenario: show actual count
            (current_count, PAGES.len())
        }
    }

    pub fn clear_all_raw_data(&mut self) {
        for config in self.files.values_mut() {
            config.clear_raw_data();
        }
    }
}

// pub struct Config {
//     pub base_url: &'static str,
// }

// impl Config {
//     pub fn new() -> Self {
//         #[cfg(debug_assertions)]
//         let base_url = "http://localhost:8080";

//         #[cfg(not(debug_assertions))]
//         let base_url = "https://smythg4.github.io/roboamo";

//         Self { base_url }
//     }
// }
