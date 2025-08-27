use crate::engine::assignment::FlowAssignment;
use crate::engine::builder::{build_assignment_plan, generate_assignments, AssignmentResult};
use crate::engine::person::{DutyStatus, Person};
use crate::engine::team::Team;
use crate::utilities::AppState;
use crate::views::ErrorDisplay;
use chrono::prelude::*;
use dioxus::prelude::*;
use itertools::Itertools;
use std::rc::Rc;

#[component]
pub fn Results() -> Element {
    // Subscribe to app state changes
    let app_state = use_context::<Signal<AppState>>();

    // Store just the raw data without the assignment plan
    let mut raw_data = use_signal(|| None::<(Vec<FlowAssignment>, Rc<Vec<Person>>, Rc<Vec<Team>>)>);

    // Add the selected date signal - default to today
    let mut selected_date = use_signal(|| chrono::Utc::now().date_naive());

    // Recompute when app state changes
    use_effect(move || {
        // Read app state to trigger recomputation on changes
        let _ = app_state();
        let _ = selected_date();

        // Generate fresh assignments
        let data = match generate_assignments(selected_date.read().clone()) {
            Ok(AssignmentResult {
                flow_assignments,
                people,
                teams,
            }) => Some((flow_assignments, people, teams)),
            Err(e) => {
                eprintln!("Error generating assignments: {:?}", e);
                None
            }
        };

        raw_data.set(data);
    });

    // Handle the case where assignments couldn't be generated
    let Some((ref flow_assignments, ref people, ref teams)) = *raw_data.read() else {
        return rsx! {
            div {
                class: "results-container",
                div {
                    class: "section-card",
                    h2 {
                        class: "section-title-warning",
                        "⚠️ Error Generating Assignments"
                    }
                    p {
                        class: "text-gray-600",
                        "There was an error processing the uploaded files. Please check that all files are properly formatted and try again."
                    }
                }
            }
        };
    };

    // Build the assignment plan here, where we can use the references
    let assignments = match build_assignment_plan(people, teams, flow_assignments) {
        Ok(plan) => plan,
        Err(e) => {
            //log::error!("Failed to build assignment plan: {}", e);
            // Return empty plan or show error
            return rsx! {
                ErrorDisplay {
                    error: format!("Failed to generate assignments: {}", e),
                    retry: None
                }
            };
        }
    };

    let teams_with_assignments: Vec<_> = teams
        .iter()
        .map(|team| {
            let team_assignments: Vec<_> = assignments
                .assignments
                .iter()
                .filter(|a| a.team_name == team.name)
                .collect();
            (team, team_assignments)
        })
        //.filter(|(_, team_assignments)| !team_assignments.is_empty())
        .collect::<Vec<_>>()
        .into_iter()
        .sorted_by_key(|(_, team_assignments)| team_assignments.len())
        .collect();
    let today = chrono::Utc::now().date_naive();
    rsx! {
    div {
        class: "results-container",

        // Header with summary stats
        div {
            class: "results-header",
            h1 {
                class: "results-title",
                "Assignment Results"
            }
            div {
                class: "stats-grid",
                div {
                    class: "stat-card-assigned",
                    h3 { class: "stat-number-green", "{assignments.assignments.len()}" }
                    p { class: "stat-label-green", "People Assigned" }
                }
                div {
                    class: "stat-card-unassigned",
                    h3 { class: "stat-number-yellow", "{assignments.unassigned_people.len()}" }
                    p { class: "stat-label-yellow", "Unassigned" }
                }
                div {
                    class: "stat-card-unfilled",
                    h3 { class: "stat-number-red", "{assignments.unfilled_positions.len()}" }
                    p { class: "stat-label-red", "Unfilled Positions" }
                }
            }
        }

        div {
            class: "flex items-center gap-4 mb-4",
            label {
                class: "text-sm font-medium text-gray-700",
                span { "📅 " }
                "Analysis Date:"
            }
            input {
                r#type: "date",
                class: "border border-gray-300 rounded px-3 py-1",
                value: "{selected_date().format(\"%Y-%m-%d\")}",
                onchange: move |evt| {
                    if let Ok(new_date) = chrono::NaiveDate::parse_from_str(&evt.value(), "%Y-%m-%d") {
                        selected_date.set(new_date);
                    }
                }
            }
        }

        // Assignments by Team
        div {
            class: "section-card",
            h2 {
                class: "section-title",
                "Assignments by Team"
            }
            div {
                class: "teams-grid",
                for (team, team_assignments) in teams_with_assignments {
                    div {
                        class: "team-card",
                        h3 {
                            class: "team-header",
                            span { class: "team-icon", "👥" }
                            "{team.name} ({team_assignments.len()} assigned)"
                        }

                        div {
                            class: "table-wrapper",
                            table {
                                class: "results-table",
                                thead {
                                    class: "table-header",
                                    tr {
                                        th { class: "table-header-cell", "Name" }
                                        th { class: "table-header-cell", "Rate/Rank" }
                                        th { class: "table-header-cell", "Role" }
                                        th { class: "table-header-cell", "Status" }
                                        th { class: "table-header-cell", "PRD" }
                                    }
                                }
                                tbody {
                                    for assignment in team_assignments {
                                        tr {
                                            class: "table-row",
                                            td {
                                                class: "table-cell-name",
                                                "{assignment.person.name}"
                                            }
                                            td {
                                                class: "table-cell-muted",
                                                "{assignment.person.raterank}"
                                            }
                                            td {
                                                class: "table-cell",
                                                span {
                                                    class: "role-badge",
                                                    "{assignment.qualification}"
                                                }
                                            }
                                            td {
                                                class: "table-cell",
                                                match assignment.person.duty_status {
                                                    DutyStatus::TAR => rsx! {
                                                        span {
                                                            class: "status-badge-tar",
                                                            "TAR"
                                                        }
                                                    },
                                                    DutyStatus::SELRES => rsx! {
                                                        span {
                                                            class: "status-badge-selres",
                                                            "SELRES"
                                                        }
                                                    }
                                                }
                                            }
                                            td {
                                                class: "table-cell-muted",
                                                if let Some(prd) = assignment.person.prd {
                                                    span {
                                                        class: get_prd_css_class(prd, today),
                                                        "{prd}"
                                                    }
                                                } else {
                                                    "-"
                                                }
                                            }
                                        }
                                    }
                                    // rows for missing quals
                                    for missing in assignments.unfilled_positions.iter()
                                        .filter(|(team_name,_)| team_name == &team.name) {
                                            tr {
                                                class: "table-row bg-red-50",
                                                td {
                                                    class: "table-cell-name text-red-600",
                                                    span { class: "text-xl mr-2", "⚠️" }
                                                    //span { class: "font-semibold", "MISSING" }
                                                }
                                                td { class: "table-cell-muted text-red-400", "" }
                                                td {
                                                    class: "table-cell",
                                                    span {
                                                        class: "role-badge bg-red-100 text-red-800",
                                                        "{missing.1}"
                                                    }
                                                }
                                                td { class: "table-cell-muted text-red-400", "" }
                                                td { class: "table-cell-muted text-red-400", "" }
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

        // Unassigned People
        if !assignments.unassigned_people.is_empty() {
            div {
                class: "section-card",
                h2 {
                    class: "section-title-alert",
                    "👤 Unassigned Personnel"
                }

                div {
                    class: "table-wrapper",
                    table {
                        class: "results-table",
                        thead {
                            class: "table-header",
                            tr {
                                th { class: "table-header-cell", "Name" }
                                th { class: "table-header-cell", "Rate/Rank" }
                                th { class: "table-header-cell", "Status" }
                                th { class: "table-header-cell", "PRD" }
                                th { class: "table-header-cell", "Eligible Roles" }
                            }
                        }
                        tbody {
                            for person in assignments.unassigned_people.iter()
                                .sorted_by(|p, q| Ord::cmp(&q.qualifications.len(), &p.qualifications.len())) {
                                tr {
                                    class: "table-row",
                                    td {
                                        class: "table-cell-name",
                                        "{person.name}"
                                    }
                                    td {
                                        class: "table-cell-muted",
                                        "{person.raterank}"
                                    }
                                    td {
                                        class: "table-cell",
                                        match person.duty_status {
                                            DutyStatus::TAR => rsx! {
                                                span {
                                                    class: "status-badge-tar",
                                                    "TAR"
                                                }
                                            },
                                           DutyStatus::SELRES => rsx! {
                                                span {
                                                    class: "status-badge-selres",
                                                    "SELRES"
                                                }
                                            },
                                        }

                                    }
                                    td {
                                        class: "table-cell",
                                        match person.get_prd() {
                                            Some(date) => rsx! {
                                                span {
                                                    class: "table-cell-muted",
                                                    "{date}",
                                                }
                                            },
                                            None => rsx! {
                                                span {
                                                    class: "table-cell-muted",
                                                    "-",
                                                }
                                            }
                                        }
                                    }
                                    td {
                                        class: "table-cell-small",
                                        "{person.qualifications.join(\", \")}"
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

fn get_prd_css_class(prd: chrono::NaiveDate, today: chrono::NaiveDate) -> &'static str {
    let months_remaining =
        (prd.year() - today.year()) * 12 + (prd.month() as i32 - today.month() as i32);

    if months_remaining >= 12 {
        "text-gray-600"
    } else if months_remaining >= 6 {
        "text-yellow-600 font-semibold"
    } else {
        "text-orange-600 font-bold"
    }
}
