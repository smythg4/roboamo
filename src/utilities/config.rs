use crate::utilities::parsing::{PRDList, PersonnelQuals, QualTable, Teams};
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
    PreviewType::ASM,
    PreviewType::FLTMPS,
];
const FILE_TYPES: [&str; 4] = [".csv", ".csv", ".xlsx", ".xlsx"];
const NEXT_PAGES: [Option<&str>; 4] = [Some("Qual Defs"), Some("ASM"), Some("FLTMPS"), None];
const DEMO_PATHS: [Option<&str>; 4] = [
    Some("/assets/demo/demoteams.csv"),
    Some("/assets/demo/demoqualtable.csv"),
    Some("/assets/demo/demoasm.xlsx"),
    Some("/assets/demo/demofltmps.xlsx"),
];

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PreviewType {
    Requirements,
    QualDef,
    ASM,
    FLTMPS,
}

#[derive(Debug, Clone)]
pub enum ParsedData {
    Requirements(Rc<Teams>),
    QualDefs(Rc<QualTable>),
    ASM(Rc<PersonnelQuals>),
    FLTMPS(Rc<PRDList>),
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

#[derive(Debug, Clone)]
pub struct AppState {
    pub files: HashMap<String, FileUploadConfig>,
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
        AppState { files }
    }
}

impl AppState {
    pub fn all_files_uploaded(&self) -> bool {
        self.files.values().all(|f| f.parsed_data.is_some())
    }

    pub fn upload_progress(&self) -> (usize, usize) {
        (
            self.files
                .values()
                .filter(|f| f.parsed_data.is_some())
                .count(),
            PAGES.len(),
        )
    }
}

pub struct Config {
    pub base_url: &'static str,
}

impl Config {
    pub fn new() -> Self {
        #[cfg(debug_assertions)]
        let base_url = "http://localhost:8080";

        #[cfg(not(debug_assertions))]
        let base_url = "https://smythg4.github.io/roboamo";

        Self { base_url }
    }
}
