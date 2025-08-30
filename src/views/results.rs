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
    person::Person,
    team::{Position, Team},
};

// Local crate imports - other
use crate::{
    components::{AssignmentStats, InteractionMode, PlayerCard, TeamCard, UnassignedTable},
    utilities::AppState,
};

// Context for shared assignment UI state
#[derive(Clone)]
pub struct AssignmentUIContext {
    pub interaction_mode: Signal<InteractionMode>,
    pub selected_assignments: Signal<Vec<(String, Option<String>, Option<Position>)>>,
    pub people: ReadOnlySignal<Rc<Vec<Person>>>, // for eligibility calculations
}

#[component]
pub fn Results() -> Element {
    // Local UI state (not in context)
    let mut hovered_person = use_signal(|| None::<(Person, Option<String>)>); // (person, current assignment)
    let mut mouse_position = use_signal(|| (0.0, 0.0));
    let mut selected_date = use_signal(|| chrono::Utc::now().date_naive());
    let mut persistent_locks = use_signal(HashMap::<(String, Position), String>::new);

    // Subscribe to app state changes
    let app_state = use_context::<Signal<AppState>>();

    // Raw data storage
    let mut raw_data = use_signal(|| None::<(Vec<FlowAssignment>, Rc<Vec<Person>>, Rc<Vec<Team>>)>);

    // Context state - these will be provided to child components
    let mut interaction_mode = use_signal(|| InteractionMode::ViewOnly);
    let mut selected_assignments =
        use_signal(Vec::<(String, Option<String>, Option<Position>)>::new);
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
        let data = match generate_assignments(selected_date(), all_locks, &app_state.read()) {
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

    // Build assignment plan using a memoized signal to avoid ownership issues
    let assignments = use_memo(move || {
        let raw_data_current = raw_data.read();
        let Some((ref flow_assignments, ref people, ref teams)) = *raw_data_current else {
            return None;
        };

        match build_assignment_plan(people, teams, flow_assignments) {
            Ok(plan) => Some(plan),
            Err(_e) => None, // TODO: better error handling
        }
    });

    // Create the people signal for context
    let people_signal = use_memo(move || {
        let raw_data_current = raw_data.read();
        match raw_data_current.as_ref() {
            Some((_flow_assignments, people, _teams)) => people.clone(),
            None => Rc::new(Vec::new()),
        }
    });

    // Create the context
    let ui_context = AssignmentUIContext {
        interaction_mode,
        selected_assignments,
        people: people_signal.into(),
    };

    // Check if assignments were generated successfully
    if assignments.read().is_none() {
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
    }

    // Memoize unassigned people - only recalculates when assignments change
    let unassigned_people = use_memo(move || {
        let assignments_current = assignments.read();
        let Some(ref assignments_plan) = *assignments_current else {
            return Vec::new();
        };

        assignments_plan
            .unassigned_people
            .iter()
            .sorted_by(|p, q| Ord::cmp(&q.qualifications.len(), &p.qualifications.len()))
            .cloned()
            .collect::<Vec<_>>()
    });

    // Memoize teams sorted by assignment count
    let teams_sorted = use_memo(move || {
        let assignments_current = assignments.read();
        let Some(ref assignments_plan) = *assignments_current else {
            return Vec::new();
        };
        let raw_data_current = raw_data.read();
        let Some((_flow_assignments, _people, teams)) = raw_data_current.as_ref() else {
            return Vec::new();
        };

        teams
            .iter()
            .map(|team| {
                let assignment_count = assignments_plan
                    .assignments
                    .iter()
                    .filter(|a| a.team_name == team.name)
                    .count();
                (team, assignment_count)
            })
            .sorted_by_key(|(_, count)| *count)
            .map(|(team, _)| team.clone())
            .collect::<Vec<_>>()
    });

    // Event handlers - these will be passed as props, not in context
    let on_selection_change = move |((person_name, team, position), is_checked): (
        (String, Option<String>, Option<Position>),
        bool,
    )| {
        let assignment_id = (person_name, team, position);
        let new_selections = toggle_assignment_selection(
            assignment_id,
            is_checked,
            selected_assignments(),
            interaction_mode(),
        );
        selected_assignments.set(new_selections);
    };

    let on_person_hover =
        move |(person, assignment, coords): (Person, Option<String>, (f64, f64))| {
            hovered_person.set(Some((person, assignment)));
            mouse_position.set(coords);
        };

    let on_person_leave = move |_| {
        hovered_person.set(None);
    };

    use_context_provider(|| ui_context);

    rsx! {
        div {
            class: "results-container",

        // Header with summary stats
        AssignmentStats {
            assignments_signal: assignments.read().clone().unwrap(),
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
                for team in teams_sorted() {
                    TeamCard {
                        team: team.clone(),
                        assignments_signal: assignments.read().clone().unwrap(),
                        analysis_date_signal: selected_date,
                        on_selection_change: on_selection_change,
                        on_person_hover: on_person_hover,
                        on_person_leave: on_person_leave,
                    }
                }
            }
        }

        // Unassigned Personnel
        if !unassigned_people().is_empty() {
            UnassignedTable {
                assignments_signal: assignments.read().clone().unwrap(),
                analysis_date_signal: selected_date,
                on_selection_change: on_selection_change,
                on_person_hover: on_person_hover,
                on_person_leave: on_person_leave,
            }
        }

        if let Some((person, assignment)) = hovered_person() {
            PlayerCard {
                person,
                current_assignment: assignment,
                position: mouse_position(),
            }
        }
    } // Close main div
    } // Close rsx! block
}

fn should_add_selection(interaction_mode: InteractionMode, current_count: usize) -> bool {
    match interaction_mode {
        InteractionMode::Swap => current_count < 2,
        InteractionMode::Lock => true,
        InteractionMode::ViewOnly => false,
    }
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
