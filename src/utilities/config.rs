
use std::collections::HashMap;
use std::rc::Rc;
use crate::utilities::parsing::{Teams, QualTable, PersonnelQuals, PRDList};

pub const PAGES: [&str; 4] = [
    "Requirements",
    "Qual Defs",
    "ASM",
    "FLTMPS"];
    
const DESCS: [&str; 4] = [
    "Details the groups along with the qualifications and quantity of each qualification required.",
    "A lookup table to convert ASM qualifications into common parlance.",
    "Export from ASM (MMP) detailing squadron members and their qualifications.",
    "FLTMPS export to look up PRDs for TAR sailors."];

const PREVIEWS: [PreviewType; 4] = [
    PreviewType::Requirements,
    PreviewType::QualDef,
    PreviewType::ASM,
    PreviewType::FLTMPS,
];
const FILE_TYPES: [&str; 4] = [
    ".csv",
    ".csv",
    ".xlsx",
    ".xlsx",
];
const NEXT_PAGES: [Option<&str>; 4] = [
    Some("Qual Defs"),
    Some("ASM"),
    Some("FLTMPS"),
    None,
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
            };
            files.entry(page_name.to_string())
                .or_insert(file);
        }
        AppState { files }
    }
}

impl AppState {
    pub fn all_files_uploaded(&self) -> bool {
        self.files.values().all(|f| f.parsed_data.is_some())
    }

    pub fn upload_progress(&self) -> (usize, usize) {
        ( self.files.values().filter(|f| f.parsed_data.is_some() ).count(), PAGES.len() )
    }
}