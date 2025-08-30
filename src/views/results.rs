// Standard library imports
use std::collections::HashMap;
use std::rc::Rc;

// External crate imports
use chrono::prelude::*;
use dioxus::prelude::*;
use itertools::Itertools;

// Local crate imports - engine
use crate::engine::{
    assignment::{AssignmentLock, FlowAssignment},
    builder::{build_assignment_plan, generate_assignments, AssignmentResult},
    person::{DutyStatus, Person},
    team::{Position, Team},
};

// Local crate imports - other
use crate::{components::SearchBar, utilities::AppState, views::ErrorDisplay};

#[derive(Clone, Copy, PartialEq)]
enum InteractionMode {
    ViewOnly,
    Swap,
    Lock,
}

#[component]
pub fn Results() -> Element {
    let mut search_query = use_signal(String::new);
    // Subscribe to app state changes
    let app_state = use_context::<Signal<AppState>>();
    let mut interaction_mode = use_signal(|| InteractionMode::ViewOnly);
    // Store just the raw data without the assignment plan
    let mut raw_data = use_signal(|| None::<(Vec<FlowAssignment>, Rc<Vec<Person>>, Rc<Vec<Team>>)>);

    // Add the selected date signal - default to today
    let mut selected_date = use_signal(|| chrono::Utc::now().date_naive());
    let mut selected_assignments =
        use_signal(Vec::<(String, Option<String>, Option<Position>)>::new); // (person_name, team_name, position)

    let mut persistent_locks = use_signal(HashMap::<(String, Position), String>::new);
    use_effect(move || {
        // Read app state to trigger recomputation on changes
        let _ = app_state();
        let _ = selected_date();
        let current_persistent_locks = persistent_locks();

        let all_locks = if !current_persistent_locks.is_empty() {
            let assignment_locks = current_persistent_locks
                .iter()
                .map(|((team_name, position), person)| AssignmentLock {
                    person_name: person.clone(),
                    team_name: Some(team_name.clone()),
                    position: Some(position.clone()),
                })
                .collect();
            Some(assignment_locks)
        } else {
            None
        };

        // Generate fresh assignments
        let data = match generate_assignments(selected_date(), all_locks) {
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

    let unassigned_people: Vec<_> = assignments
        .unassigned_people
        .iter()
        .sorted_by(|p, q| Ord::cmp(&q.qualifications.len(), &p.qualifications.len()))
        .cloned()
        .collect();

    let teams_with_assignments: Vec<_> = teams
        .iter()
        .map(|team| {
            let team_assignments: Vec<_> = assignments
                .assignments
                .iter().filter(|&a| a.team_name == team.name).cloned()
                .sorted_by_key(|a| !a.manual_override)
                .collect();
            (team, team_assignments)
        })
        //.filter(|(_, team_assignments)| !team_assignments.is_empty())
        .sorted_by_key(|(_, team_assignments)| team_assignments.len())
        .collect();
    let today = selected_date();

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
            class: "sticky top-17 z-50 bg-white shadow-md border-b border-gray-200 flex gap-2 p-4 w-175",
            button {
                class: match interaction_mode() {
                    InteractionMode::ViewOnly => "px-4 py-2 bg-gray-600 text-white rounded-lg font-medium",
                    _ => "px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
                },
                onclick: move |_| {
                    interaction_mode.set(InteractionMode::ViewOnly);
                    selected_assignments.set(vec![]);
                },
                "👁️ View Only"
            }
            // Swap Mode - button changes when selections are ready
            button {
                class: match (interaction_mode(), selected_assignments().len()) {
                    (InteractionMode::Swap, 2) => "px-4 py-2 bg-blue-600 text-white rounded-lg font-medium animate-pulse",
                    (InteractionMode::Swap, _) => "px-4 py-2 bg-blue-500 text-white rounded-lg font-medium",
                    _ => "px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300",
                },
                onclick: move |_| {
                    if interaction_mode() == InteractionMode::Swap && selected_assignments().len() == 2 {
                        // Execute swap
                        let selections = selected_assignments();
                        let (person1, team1, pos1) = &selections[0];
                        let (person2, team2, pos2) = &selections[1];

                        persistent_locks.with_mut(|locks| {
                            if let (Some(team), Some(pos)) = (team2, pos2) {
                                locks.insert((team.clone(), pos.clone()), person1.clone());
                            }
                            if let (Some(team), Some(pos)) = (team1, pos1) {
                                locks.insert((team.clone(), pos.clone()), person2.clone());
                            }
                        });
                        selected_assignments.set(vec![]);
                        interaction_mode.set(InteractionMode::ViewOnly);
                    } else {
                        interaction_mode.set(InteractionMode::Swap);
                        selected_assignments.set(vec![]);
                    }
                },
                match (interaction_mode(), selected_assignments().len()) {
                    (InteractionMode::Swap, 2) => "🔄 Execute Swap",
                    (InteractionMode::Swap, 1) => "🔄 Select One More",
                    (InteractionMode::Swap, _) => "🔄 Swap Mode",
                    _ => "🔄 Swap Mode"
                }
            }

            // Lock Mode - button changes when selections exist
            button {
                class: match (interaction_mode(), selected_assignments().len()) {
                    (InteractionMode::Lock, n) if n > 0 => "px-4 py-2 bg-orange-600 text-white rounded-lg font-medium animate-pulse",
                    (InteractionMode::Lock, _) => "px-4 py-2 bg-orange-500 text-white rounded-lg font-medium",
                    _ => "px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
                },
                onclick: move |_| {
                    if interaction_mode() == InteractionMode::Lock && !selected_assignments().is_empty() {
                        // Execute locks
                        let selections = selected_assignments();
                        persistent_locks.with_mut(|locks| {
                            for (person, team, pos) in selections {
                                if let (Some(team), Some(pos)) = (team, pos) {
                                    locks.insert((team, pos), person);
                                }
                            }
                        });
                        selected_assignments.set(vec![]);
                        interaction_mode.set(InteractionMode::ViewOnly);
                    } else {
                        interaction_mode.set(InteractionMode::Lock);
                        selected_assignments.set(vec![]);
                    }
                },
                match (interaction_mode(), selected_assignments().len()) {
                    (InteractionMode::Lock, n) if n > 0 => format!("🔒 Lock {} Assignments", n),
                    (InteractionMode::Lock, _) => "🔒 Lock Mode".to_string(),
                    _ => "🔒 Lock Mode".to_string()
                }
            }

            // button to clear all locked selections
            button {
                class: "px-3 py-1 bg-red-500 text-white rounded text-sm hover:bg-red-600",
                onclick: move |_| {
                    persistent_locks.set(HashMap::new());
                },
                "Clear All Locks ({persistent_locks().len()})"
            }
        }
        div {
            class: "sticky top-40 z-50 bg-white shadow-md border-b border-gray-200 flex items-center gap-2 p-4 w-175",
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
                                        th { class: "table-header-cell", "" }
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
                                            class: {if is_assignment_selected(
                                                &selected_assignments(),
                                                &assignment.person.name,
                                                &assignment.team_name,
                                                &assignment.position,
                                            ) {
                                                "table-row bg-yellow-50 border-l-4 border-l-yellow-400"
                                            } else if is_swap_eligible(
                                        interaction_mode(),
                                        &selected_assignments(),
                                        &assignment.person.name,
                                        Some(assignment.team_name.as_str()),
                                        Some(&assignment.position),
                                        people,
                                    ) {
                                        "table-row bg-emerald-100 border-l-4 border-l-emerald-400"
                                    } else {
                                        "table-row"
                                    }},
                                            td {
                                                class: "table-cell-muted",
                                                //if interaction_mode() != InteractionMode::ViewOnly {
                                                    input {
                                                        r#type: "checkbox",
                                                        style: if interaction_mode() == InteractionMode::ViewOnly {
                                                            "visibility: hidden;"
                                                        } else {
                                                            "visibility: visible;"
                                                        },
                                                        checked: is_assignment_selected(
                                                            &selected_assignments(),
                                                            &assignment.person.name,
                                                            &assignment.team_name,
                                                            &assignment.position,
                                                        ),
                                                        disabled: {
                                                            let current_selected = selected_assignments();
                                                            let is_currently_selected = is_assignment_selected(
                                                                &current_selected,
                                                                &assignment.person.name,
                                                                &assignment.team_name,
                                                                &assignment.position,
                                                            );
                                                            is_checkbox_disabled(interaction_mode(), current_selected.len(), is_currently_selected)
                                                        },
                                                        onchange: move |evt| {
                                                            let assignment_id = (
                                                                assignment.person.name.clone(),
                                                                Some(assignment.team_name.clone()),
                                                                Some(assignment.position.clone()),
                                                            );
                                                            let new_selections = toggle_assignment_selection(
                                                                assignment_id,
                                                                evt.checked(),
                                                                selected_assignments(),
                                                                interaction_mode(),
                                                            );
                                                            selected_assignments.set(new_selections);
                                                        }
                                                    }
                                                //}
                                            }
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
                                                    class: if assignment.manual_override {
                                                        "role-badge role-badge--locked"
                                                    } else {
                                                        "role-badge"
                                                    },
                                                    "{assignment.position.qualification}"
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
                                                    ""
                                                }
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
        if !unassigned_people.is_empty() {
            div {
                class: "section-card",
                h2 {
                    class: "section-title-alert",
                    "👤 Unassigned Personnel"
                }
                SearchBar {
                    placeholder: "Search name or qual...",
                    value: search_query(),
                    onchange: move |value| search_query.set(value),
                }
                div {
                    class: "table-wrapper",
                    table {
                        class: "results-table",
                        thead {
                            class: "table-header",
                            tr {
                                th { class: "table-header-cell", "" }
                                th { class: "table-header-cell", "Name" }
                                th { class: "table-header-cell", "Rate/Rank" }
                                th { class: "table-header-cell", "Status" }
                                th { class: "table-header-cell", "PRD" }
                                th { class: "table-header-cell", "Eligible Roles" }
                            }
                        }
                        tbody {
                            for person in unassigned_people.iter().cloned()
                                .filter(|p| {
                                        let query = search_query().to_lowercase();
                                        query.is_empty() ||
                                        p.name.to_lowercase().contains(&query) ||
                                        p.qualifications.iter().any(|q| q.to_lowercase().contains(&query))
                                    }) {
                                tr {
                                    class: if is_swap_eligible(
                                        interaction_mode(),
                                        &selected_assignments(),
                                        person.get_name(),
                                        None,
                                        None,
                                        people,
                                    ) {
                                        "table-row bg-emerald-100 border-l-4 border-l-emerald-400"
                                    } else {
                                        "table-row"
                                    },
                                    td {
                                        class: "table-cell-muted",
                                        input {
                                            r#type: "checkbox",
                                            style: if interaction_mode() == InteractionMode::ViewOnly {
                                                "visibility: hidden;"
                                            } else {
                                                "visibility: visible;"
                                            },
                                            checked: is_unassigned_selected(
                                                  &selected_assignments(),
                                                  &person.name,
                                              ),
                                            disabled: {
                                                  let current_selected = selected_assignments();
                                                  let is_currently_selected = is_unassigned_selected(
                                                      &current_selected,
                                                      &person.name,
                                                  );
                                                  is_checkbox_disabled(interaction_mode(), current_selected.len(), is_currently_selected)
                                              },
                                            onchange: move |evt| {
                                                  let assignment_id = (person.name.clone(), None, None);
                                                  let new_selections = toggle_assignment_selection(
                                                      assignment_id,
                                                      evt.checked(),
                                                      selected_assignments(),
                                                      interaction_mode(),
                                                  );
                                                  selected_assignments.set(new_selections);
                                          }
                                        }
                                    }
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
                                        "{person.qualifications.iter().sorted().join(\", \")}"
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

fn should_add_selection(interaction_mode: InteractionMode, current_count: usize) -> bool {
    match interaction_mode {
        InteractionMode::Swap => current_count < 2,
        InteractionMode::Lock => true,
        InteractionMode::ViewOnly => false,
    }
}

fn is_checkbox_disabled(
    interaction_mode: InteractionMode,
    current_count: usize,
    is_currently_selected: bool,
) -> bool {
    match interaction_mode {
        InteractionMode::Swap => current_count >= 2 && !is_currently_selected,
        InteractionMode::Lock => false,
        InteractionMode::ViewOnly => true,
    }
}

fn is_assignment_selected(
    selections: &[(String, Option<String>, Option<Position>)],
    person_name: &str,
    team_name: &str,
    position: &Position,
) -> bool {
    selections.iter().any(|(name, team, pos)| {
        name == person_name && team.as_deref() == Some(team_name) && pos.as_ref() == Some(position)
    })
}

fn toggle_assignment_selection(
    assignment_id: (String, Option<String>, Option<Position>),
    is_checked: bool,
    current_selections: Vec<(String, Option<String>, Option<Position>)>,
    interaction_mode: InteractionMode,
) -> Vec<(String, Option<String>, Option<Position>)> {
    if is_checked {
        let mut updated = current_selections;
        if should_add_selection(interaction_mode, updated.len()) {
            updated.push(assignment_id);
        }
        updated
    } else {
        current_selections
            .into_iter()
            .filter(|item| item != &assignment_id)
            .collect()
    }
}

fn is_unassigned_selected(
    selections: &[(String, Option<String>, Option<Position>)],
    person_name: &str,
) -> bool {
    selections
        .iter()
        .any(|(name, team, pos)| name == person_name && team.is_none() && pos.is_none())
}

fn is_swap_eligible(
    interaction_mode: InteractionMode,
    current_selections: &[(String, Option<String>, Option<Position>)],
    assignment_person: &str,
    assignment_team: Option<&str>,
    assignment_position: Option<&Position>,
    people: &[Person],
) -> bool {
    // Only relevant in swap mode
    if interaction_mode != InteractionMode::Swap {
        return false;
    }

    // Need exactly 1 person selected to show eligibility
    if current_selections.len() != 1 {
        return false;
    }

    // Don't highlight the currently selected person
    let is_currently_selected = match (assignment_team, assignment_position) {
        (Some(team), Some(pos)) => {
            is_assignment_selected(current_selections, assignment_person, team, pos)
        }
        _ => {
            // For unassigned people, check if they're selected as unassigned
            current_selections.iter().any(|(name, team, pos)| {
                name == assignment_person && team.is_none() && pos.is_none()
            })
        }
    };

    if is_currently_selected {
        return false;
    }

    // Get the selected person and their position
    let (selected_person_name, _selected_team, selected_position) = &current_selections[0];
    let Some(selected_position) = selected_position else {
        return false;
    };

    // Find both people in the people list
    let Some(selected_person) = people.iter().find(|p| &p.name == selected_person_name) else {
        return false;
    };
    let Some(target_person) = people.iter().find(|p| p.name == assignment_person) else {
        return false;
    };

    if assignment_position.is_none() {
        // This is an unassigned person - only check if they can do the selected job
        return target_person
            .qualifications
            .contains(&selected_position.qualification);
    }

    // For assigned people, check mutual qualification
    let assignment_position = assignment_position.unwrap(); // Safe because we checked is_none() above

    // Check mutual qualification:
    // 1. Can selected person do target's job?
    let can_selected_do_target = selected_person
        .qualifications
        .contains(&assignment_position.qualification);
    // 2. Can target person do selected's job?
    let can_target_do_selected = target_person
        .qualifications
        .contains(&selected_position.qualification);

    can_selected_do_target && can_target_do_selected
}
