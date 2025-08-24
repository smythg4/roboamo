use dioxus::prelude::*;

use crate::utilities::{parse_requirements, parse_qual_defs, parse_asm_file, parse_fltmps_file};
use crate::utilities::PreviewType;
use crate::utilities::AppState;

use std::rc::Rc;

#[component]
pub fn Preview(preview_type: PreviewType) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    
    // Find the file data for the current page
    let file_data = app_state().files.values()
        .find(|f| f.preview_type == preview_type)
        .and_then(|f| f.file_content.clone());
    
    rsx! {
        div {
            class: "preview-section",
            {
                if let Some(data) = file_data {
                    let data = data.clone();
                    match preview_type {
                        PreviewType::Requirements => rsx!{ RequirementsPreview { data } },
                        PreviewType::QualDef => rsx!{ QualDefPreview { data } },
                        PreviewType::ASM => rsx!{ ASMPreview { data } },
                        PreviewType::FLTMPS => rsx!{ FLTMPSPreview { data } },
                    }
                } else {
                    rsx! { 
                        div {
                            class: "flex items-center justify-center h-64 text-gray-500",
                            div {
                                class: "text-center",
                                svg {
                                    class: "w-16 h-16 mx-auto mb-4 text-gray-300",
                                    fill: "none",
                                    stroke: "currentColor",
                                    "viewBox": "0 0 24 24",
                                    path {
                                        "stroke-linecap": "round",
                                        "stroke-linejoin": "round",
                                        "stroke-width": "2",
                                        d: "M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"
                                    }
                                }
                                p { 
                                    class: "text-lg font-medium",
                                    "No file uploaded yet" 
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn RequirementsPreview(data: Rc<Vec<u8>>) -> Element {
    match parse_requirements(data) {
        Ok(teams) => {
            rsx! {
                div {
                    class: "space-y-6",
                    
                    // Summary card
                    div {
                        class: "bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg p-4 border border-blue-200",
                        div {
                            class: "flex items-center justify-between",
                            div {
                                h3 {
                                    class: "text-lg font-semibold text-gray-900",
                                    "Requirements Summary"
                                }
                                p {
                                    class: "text-sm text-gray-600 mt-1",
                                    "{teams.len()} teams configured"
                                }
                            }
                            div {
                                class: "text-3xl",
                                "📋"
                            }
                        }
                    }
                    
                    // Teams grid
                    div {
                        class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                        for (team_name, requirements) in teams {
                            div {
                                class: "bg-white rounded-lg shadow-sm border border-gray-200 hover:shadow-md transition-shadow duration-200",
                                div {
                                    class: "bg-gradient-to-r from-gray-50 to-gray-100 px-4 py-3 border-b border-gray-200",
                                    h4 { 
                                        class: "font-semibold text-gray-900 flex items-center gap-2",
                                        span { "👥" }
                                        "{team_name}"
                                    }
                                }
                                div {
                                    class: "p-4",
                                    if requirements.is_empty() {
                                        p {
                                            class: "text-sm text-gray-500 italic",
                                            "No requirements defined"
                                        }
                                    } else {
                                        div {
                                            class: "space-y-2",
                                            for req in requirements {
                                                div {
                                                    class: "flex justify-between items-center py-1.5 px-2 rounded hover:bg-gray-50",
                                                    span {
                                                        class: "text-sm font-medium text-gray-700",
                                                        "{req.qual_name}"
                                                    }
                                                    span {
                                                        class: "inline-flex items-center justify-center min-w-[2rem] px-2 py-0.5 text-xs font-bold text-blue-700 bg-blue-100 rounded-full",
                                                        "{req.qual_qty}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        Err(e) => {
            rsx! {
                div {
                    class: "bg-red-50 border border-red-200 rounded-lg p-4",
                    div {
                        class: "flex items-start",
                        svg {
                            class: "w-5 h-5 text-red-600 mt-0.5 mr-3 flex-shrink-0",
                            fill: "currentColor",
                            "viewBox": "0 0 20 20",
                            path {
                                "fill-rule": "evenodd",
                                d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z",
                                "clip-rule": "evenodd"
                            }
                        }
                        div {
                            h3 {
                                class: "text-sm font-medium text-red-800",
                                "Error reading file"
                            }
                            p {
                                class: "text-sm text-red-700 mt-1",
                                "{e}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn QualDefPreview(data: Rc<Vec<u8>>) -> Element {
    match parse_qual_defs(data) {
        Ok(quals) => {
            rsx! {
                div {
                    class: "space-y-4",
                    
                    // Summary card
                    div {
                        class: "bg-gradient-to-r from-green-50 to-emerald-50 rounded-lg p-4 border border-green-200",
                        div {
                            class: "flex items-center justify-between",
                            div {
                                h3 {
                                    class: "text-lg font-semibold text-gray-900",
                                    "Qualification Definitions"
                                }
                                p {
                                    class: "text-sm text-gray-600 mt-1",
                                    "{quals.len()} qualifications mapped"
                                }
                            }
                            div {
                                class: "text-3xl",
                                "🔄"
                            }
                        }
                    }
                    
                    // Qualifications table
                    div {
                        class: "bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden",
                        table {
                            class: "min-w-full divide-y divide-gray-200",
                            thead {
                                class: "bg-gray-50",
                                tr {
                                    th { 
                                        class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider",
                                        "Common Name"
                                    }
                                    th { 
                                        class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider",
                                        "ASM Equivalents" 
                                    }
                                }
                            }
                            tbody {
                                class: "bg-white divide-y divide-gray-200",
                                for (qual_name, asm_quals) in quals {
                                    tr {
                                        class: "hover:bg-gray-50 transition-colors duration-150",
                                        td { 
                                            class: "px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900",
                                            {qual_name} 
                                        }
                                        td {
                                            class: "px-6 py-4 text-sm text-gray-600",
                                            div {
                                                class: "flex flex-wrap gap-2",
                                                for qual in asm_quals {
                                                    span {
                                                        class: "inline-flex items-center px-2.5 py-0.5 rounded-md text-xs font-medium bg-blue-100 text-blue-800",
                                                        {qual}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        Err(e) => {
            rsx! {
                div {
                    class: "bg-red-50 border border-red-200 rounded-lg p-4",
                    p { 
                        class: "text-sm text-red-700",
                        "Error reading file: {e}" 
                    }
                }
            }
        }
    }
}

#[component]
pub fn ASMPreview(data: Rc<Vec<u8>>) -> Element {
    match parse_asm_file(data) {
        Ok(people) => {
            let total_people = people.len();
            let total_quals: usize = people.values().map(|quals| quals.len()).sum();
            
            rsx! {
                div {
                    class: "space-y-4",
                    
                    // Summary card
                    div {
                        class: "bg-gradient-to-r from-purple-50 to-pink-50 rounded-lg p-4 border border-purple-200",
                        div {
                            class: "flex items-center justify-between",
                            div {
                                h3 {
                                    class: "text-lg font-semibold text-gray-900",
                                    "ASM Personnel Data"
                                }
                                div {
                                    class: "flex gap-4 mt-1",
                                    p {
                                        class: "text-sm text-gray-600",
                                        span { class: "font-semibold", "{total_people}" }
                                        " personnel"
                                    }
                                    p {
                                        class: "text-sm text-gray-600",
                                        span { class: "font-semibold", "{total_quals}" }
                                        " total qualifications"
                                    }
                                }
                            }
                            div {
                                class: "text-3xl",
                                "👤"
                            }
                        }
                    }
                    
                    // Search/filter bar (placeholder for future functionality)
                    div {
                        class: "bg-white rounded-lg shadow-sm border border-gray-200 p-4",
                        input {
                            r#type: "search",
                            placeholder: "Search personnel... (coming soon)",
                            disabled: true,
                            class: "w-full px-4 py-2 text-sm border border-gray-300 rounded-lg bg-gray-50 cursor-not-allowed",
                        }
                    }
                    
                    // Personnel list
                    div {
                        class: "bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden",
                        div {
                            class: "max-h-96 overflow-auto",
                            table {
                                class: "min-w-full divide-y divide-gray-200",
                                thead {
                                    class: "bg-gray-50 sticky top-0 z-10",
                                    tr {
                                        th { 
                                            class: "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-80",
                                            "Name"
                                        }
                                        th { 
                                            class: "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider",
                                            "Qualifications" 
                                        }
                                    }
                                }
                                tbody {
                                    class: "bg-white divide-y divide-gray-200",
                                    for (name, asm_quals) in people {
                                        tr {
                                            class: "hover:bg-gray-50 transition-colors duration-150",
                                            td { 
                                                class: "px-4 py-3 text-sm font-medium text-gray-900 w-80 max-w-xs",
                                                div {
                                                    class: "truncate",
                                                    title: "{name}",
                                                    {name.clone()}
                                                }
                                            }
                                            td {
                                                class: "px-4 py-3",
                                                div {
                                                    class: "flex flex-wrap gap-1 max-w-full",
                                                    for qual in asm_quals {
                                                        span {
                                                            class: "inline-block px-1.5 py-0.5 rounded text-[10px] font-medium bg-indigo-100 text-indigo-800 max-w-[200px] truncate",
                                                            title: "{qual}",
                                                            {qual.clone()}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        Err(e) => {
            rsx! {
                div {
                    class: "bg-red-50 border border-red-200 rounded-lg p-4",
                    p { 
                        class: "text-sm text-red-700",
                        "Error reading file: {e}" 
                    }
                }
            }
        }
    }
}

#[component]
pub fn FLTMPSPreview(data: Rc<Vec<u8>>) -> Element {
    match parse_fltmps_file(data) {
        Ok(prds) => {
            let total_personnel = prds.len();
            let with_prd = prds.values().filter(|prd| prd.is_some()).count();
            let selres = total_personnel - with_prd;
            
            rsx! {
                div {
                    class: "space-y-4",
                    
                    // Summary cards
                    div {
                        class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                        div {
                            class: "bg-gradient-to-r from-blue-50 to-cyan-50 rounded-lg p-4 border border-blue-200",
                            div {
                                class: "flex items-center justify-between",
                                div {
                                    p {
                                        class: "text-sm text-gray-600",
                                        "Total Personnel"
                                    }
                                    p {
                                        class: "text-2xl font-bold text-gray-900",
                                        "{total_personnel}"
                                    }
                                }
                                span { class: "text-2xl", "👥" }
                            }
                        }
                        div {
                            class: "bg-gradient-to-r from-green-50 to-emerald-50 rounded-lg p-4 border border-green-200",
                            div {
                                class: "flex items-center justify-between",
                                div {
                                    p {
                                        class: "text-sm text-gray-600",
                                        "TAR Personnel"
                                    }
                                    p {
                                        class: "text-2xl font-bold text-gray-900",
                                        "{with_prd}"
                                    }
                                }
                                span { class: "text-2xl", "🎖️" }
                            }
                        }
                        div {
                            class: "bg-gradient-to-r from-amber-50 to-orange-50 rounded-lg p-4 border border-amber-200",
                            div {
                                class: "flex items-center justify-between",
                                div {
                                    p {
                                        class: "text-sm text-gray-600",
                                        "SELRES Personnel"
                                    }
                                    p {
                                        class: "text-2xl font-bold text-gray-900",
                                        "{selres}"
                                    }
                                }
                                span { class: "text-2xl", "📅" }
                            }
                        }
                    }
                    
                    // PRD table
                    div {
                        class: "bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden",
                        div {
                            class: "px-6 py-3 bg-gray-50 border-b border-gray-200",
                            h3 {
                                class: "text-sm font-semibold text-gray-700 uppercase tracking-wider",
                                "Projected Rotation Dates"
                            }
                        }
                        div {
                            class: "max-h-96 overflow-y-auto",
                            table {
                                class: "w-full table-fixed",
                                thead {
                                    class: "bg-gray-50",
                                    tr {
                                        th { 
                                            class: "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-1/2",
                                            "Name"
                                        }
                                        th { 
                                            class: "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-1/4",
                                            "PRD" 
                                        }
                                        th { 
                                            class: "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-1/4",
                                            "Status" 
                                        }
                                    }
                                }
                                tbody {
                                    class: "bg-white divide-y divide-gray-200 text-sm",
                                    for (name, prd) in prds {
                                        tr {
                                            class: "hover:bg-gray-50 transition-colors duration-150",
                                            td { 
                                                class: "px-4 py-2 text-gray-900 truncate",
                                                title: "{name}",
                                                {name.clone()}
                                            }
                                            td {
                                                class: "px-4 py-2 text-gray-600",
                                                if let Some(date) = prd {
                                                    span {
                                                        class: "font-mono text-sm",
                                                        {date.to_string()}
                                                    }
                                                } else {
                                                    span {
                                                        class: "text-gray-400 italic",
                                                        "—"
                                                    }
                                                }
                                            }
                                            td {
                                                class: "px-4 py-2",
                                                if prd.is_some() {
                                                    span {
                                                        class: "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-purple-100 text-purple-800",
                                                        "TAR"
                                                    }
                                                } else {
                                                    span {
                                                        class: "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800",
                                                        "SELRES"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        Err(e) => {
            rsx! {
                div {
                    class: "bg-red-50 border border-red-200 rounded-lg p-4",
                    p { 
                        class: "text-sm text-red-700",
                        "Error reading file: {e}" 
                    }
                }
            }
        }
    }
}