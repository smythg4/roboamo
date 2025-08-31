use dioxus::prelude::*;

use crate::components::SearchBar;
use crate::utilities::AppState;
use crate::utilities::PreviewType;
use crate::utilities::{parse_asm_file, parse_fltmps_file, parse_qual_defs, parse_requirements};

use std::collections::HashMap;
use std::rc::Rc;

#[component]
pub fn Preview(preview_type: PreviewType) -> Element {
    let app_state = use_context::<Signal<AppState>>();

    // Find the file data for the current page
    let file_data = app_state()
        .files
        .values()
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
                        PreviewType::Asm => rsx!{ ASMPreview { data } },
                        PreviewType::Fltmps => rsx!{ FLTMPSPreview { data } },
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
            let teams_with_counts: Vec<_> = teams
                .iter()
                .map(|team| {
                    let mut team_qual_counts = HashMap::new();
                    for pos in &team.required_positions {
                        *team_qual_counts.entry(&pos.qualification).or_insert(0) += 1;
                    }
                    let mut sorted_quals: Vec<_> = team_qual_counts.into_iter().collect();
                    sorted_quals.sort_by(|a, b| a.0.cmp(b.0));
                    (team, sorted_quals)
                })
                .collect();
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
                        for (team, qual_count) in teams_with_counts {
                            div {
                                class: "bg-white rounded-lg shadow-sm border border-gray-200 hover:shadow-md transition-shadow duration-200",
                                div {
                                    class: "bg-gradient-to-r from-gray-50 to-gray-100 px-4 py-3 border-b border-gray-200",
                                    h4 {
                                        class: "font-semibold text-gray-900 flex items-center gap-2",
                                        span { "👥" }
                                        "{team.name}"
                                    }
                                }
                                div {
                                    class: "p-4",
                                    if team.required_positions.is_empty() {
                                        p {
                                            class: "text-sm text-gray-500 italic",
                                            "No requirements defined"
                                        }
                                    } else {
                                        div {
                                            class: "space-y-2",
                                            for (qual, count) in qual_count {
                                                div {
                                                    class: "flex justify-between items-center py-1.5 px-2 rounded hover:bg-gray-50",
                                                    span {
                                                        class: "text-sm font-medium text-gray-700",
                                                        "{qual}"
                                                    }
                                                    span {
                                                        class: "inline-flex items-center justify-center min-w-[2rem] px-2 py-0.5 text-xs font-bold text-blue-700 bg-blue-100 rounded-full",
                                                        "{count}"
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
    let mut search_query = use_signal(String::new);
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

                    SearchBar {
                        placeholder: "Search qualifications...",
                        value: search_query(),
                        onchange: move |value| search_query.set(value),
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
                                for (qual_name, asm_quals) in quals.iter()
                                    .filter(|(qual_name, _)| {
                                        let query = search_query().to_lowercase();
                                        query.is_empty() || qual_name.to_lowercase().contains(&query)
                                    }) {
                                    tr {
                                        class: "hover:bg-gray-50 transition-colors duration-150",
                                        td {
                                            class: "px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900",
                                            {qual_name.clone()}
                                        }
                                        td {
                                            class: "px-6 py-4 text-sm text-gray-600",
                                            div {
                                                class: "flex flex-wrap gap-2",
                                                for qual in asm_quals {
                                                    span {
                                                        class: "inline-flex items-center px-2.5 py-0.5 rounded-md text-xs font-medium bg-blue-100 text-blue-800",
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
    let mut search_term = use_signal(String::new);

    let people_resource = use_resource(move || {
        let data = data.clone();
        async move { parse_asm_file(data) }
    });

    let people = match &*people_resource.read() {
        Some(Ok(all_people)) => {
            let term = search_term.read().clone();
            let filtered_people: Vec<_> = all_people
                .iter()
                .filter(|p| {
                    let term_lower = term.to_lowercase();
                    p.name.to_lowercase().contains(&term_lower)
                        || p.qualifications
                            .iter()
                            .any(|q| q.to_lowercase().contains(&term_lower))
                })
                .cloned()
                .collect();
            filtered_people
        }
        Some(Err(e)) => {
            return rsx! {
                div {
                    class: "bg-red-50 border border-red-200 rounded-lg p-4",
                    p {
                        class: "text-sm text-red-700",
                        "Error reading file: {e}"
                    }
                }
            }
        }
        None => return rsx! { div { "Loading..."} },
    };

    let total_people = people.len();
    let total_quals: usize = people
        .iter()
        .map(|person| person.qualifications.len())
        .sum();

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

            SearchBar {
                placeholder: "Search personnel by name or qual...",
                value: search_term(),
                onchange: move |value| search_term.set(value),
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
                            for person in people {
                                tr {
                                    class: "hover:bg-gray-50 transition-colors duration-150",
                                    td {
                                        class: "px-4 py-3 text-sm font-medium text-gray-900 w-80 max-w-xs",
                                        div {
                                            class: "truncate",
                                            title: "{person.name}",
                                            {person.name.clone()}
                                        }
                                    }
                                    td {
                                        class: "px-4 py-3",
                                        div {
                                            class: "flex flex-wrap gap-1 max-w-full",
                                            for qual in person.qualifications {
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
}

#[component]
pub fn FLTMPSPreview(data: Rc<Vec<u8>>) -> Element {
    let mut search_term = use_signal(String::new);

    let fltmps_resource = use_resource(move || {
        let data = data.clone();
        async move { parse_fltmps_file(data) }
    });

    let prds = match &*fltmps_resource.read() {
        Some(Ok(all_prds)) => {
            let term = search_term.read().clone();
            let filtered_prds: Vec<_> = all_prds
                .iter()
                .filter(|(name, _)| {
                    let term_lower = term.to_lowercase();
                    name.to_lowercase().contains(&term_lower)
                })
                .map(|(name, date)| (name.clone(), *date))
                .collect();
            filtered_prds
        }
        Some(Err(e)) => {
            return rsx! {
                div {
                    class: "bg-red-50 border border-red-200 rounded-lg p-4",
                    p {
                        class: "text-sm text-red-700",
                        "Error reading file: {e}"
                    }
                }
            }
        }
        None => return rsx! { div { "Loading..."} },
    };

    let total_personnel = prds.len();
    let with_prd = prds.iter().filter(|(_, prd)| prd.is_some()).count();
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
            SearchBar {
                placeholder: "Search personnel by name...",
                value: search_term(),
                onchange: move |value| search_term.set(value),
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
}
