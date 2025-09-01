// Standard library imports
use std::collections::HashMap;
use std::rc::Rc;

// External crate imports
use dioxus::prelude::*;
use itertools::Itertools;
#[cfg(target_arch = "wasm32")]
use {wasm_bindgen, web_sys};

// Local crate imports - engine
use crate::engine::{
    assignment::{AssignmentLock, FlowAssignment},
    builder::{build_assignment_plan, generate_assignments, generate_assignments_from_processed_data, AssignmentResult},
    person::Person,
    team::{Position, Team},
};

// Type aliases to reduce complexity in function signatures
pub type SelectionChangeHandler = Callback<((String, Option<String>, Option<Position>), bool)>;
pub type PersonHoverHandler = Callback<(Person, Option<String>, (f64, f64))>;
pub type PersonLeaveHandler = Callback<()>;
pub type RolePopupOpenHandler = Callback<(Position, String, Option<Person>, (f64, f64))>; // (position, team_name, current_person, coords)
pub type AssignmentSelection = Vec<(String, Option<String>, Option<Position>)>;

// Local crate imports - other
use crate::{
    components::{AnalysisDateBar, AssignmentStats, InteractionAction, InteractionBar, InteractionMode, PlayerCard, RolePopup, TeamCard, UnassignedTable},
    utilities::{AppState, SaveState},
};

// Context for shared assignment UI state
#[derive(Clone)]
pub struct AssignmentUIContext {
    pub interaction_mode: Signal<InteractionMode>,
    pub selected_assignments: Signal<AssignmentSelection>,
    pub people: ReadOnlySignal<Rc<Vec<Person>>>, // for eligibility calculations
}

#[component]
pub fn Results() -> Element {
    // Local UI state (not in context)
    let mut hovered_person = use_signal(|| None::<(Person, Option<String>)>); // (person, current assignment)
    let mut mouse_position = use_signal(|| (0.0, 0.0));
    let selected_date = use_signal(|| chrono::Utc::now().date_naive());
    
    // Popup state
    let mut role_popup_state = use_signal(|| None::<(Position, String, Option<Person>, (f64, f64))>); // (position, team_name, current_person, popup_position)

    // Subscribe to app state changes
    let mut app_state = use_context::<Signal<AppState>>();

    // Raw data storage
    let mut raw_data = use_signal(|| None::<(Vec<FlowAssignment>, Rc<Vec<Person>>, Rc<Vec<Team>>)>);

    // Context state - these will be provided to child components
    let mut interaction_mode = use_signal(|| InteractionMode::ViewOnly);
    let mut selected_assignments =
        use_signal(Vec::<(String, Option<String>, Option<Position>)>::new);
    use_effect(move || {
        // Read app state to trigger recomputation on changes
        let app_state_val = app_state();
        let _ = selected_date();
        let current_persistent_locks = app_state_val.persistent_locks.clone();
        
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("Results useEffect triggered - checking for data changes..."));

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
        let app_state_read = &app_state_val;
        
        // Check if we have data loaded from save state vs file uploads
        let has_fltmps = app_state_read.files.get("FLTMPS")
            .and_then(|config| config.parsed_data.as_ref())
            .is_some();
        let has_requirements = app_state_read.files.get("Requirements")
            .and_then(|config| config.parsed_data.as_ref())
            .is_some();
        let has_asm = app_state_read.files.get("ASM")
            .and_then(|config| config.parsed_data.as_ref())
            .is_some();
        
        let data = if has_requirements && has_asm && !has_fltmps {
            // This looks like save state data - use processed data directly
            let teams = app_state_read.files.get("Requirements")
                .and_then(|config| config.parsed_data.as_ref())
                .and_then(|data| match data {
                    crate::utilities::config::ParsedData::Requirements(teams) => Some(teams.as_ref().clone()),
                    _ => None,
                });
            let people = app_state_read.files.get("ASM")
                .and_then(|config| config.parsed_data.as_ref())
                .and_then(|data| match data {
                    crate::utilities::config::ParsedData::Personnel(people) => Some(people.as_ref().clone()),
                    _ => None,
                });
                
            match (people, teams) {
                (Some(people), Some(teams)) => {
                    match generate_assignments_from_processed_data(selected_date(), all_locks, people, teams) {
                        Ok(AssignmentResult { flow_assignments, people, teams }) => Some((flow_assignments, people, teams)),
                        Err(e) => {
                            eprintln!("Error generating assignments from processed data: {:?}", e);
                            None
                        }
                    }
                }
                _ => None,
            }
        } else {
            // Use normal file upload flow
            match generate_assignments(selected_date(), all_locks, &app_state_read) {
                Ok(AssignmentResult { flow_assignments, people, teams }) => Some((flow_assignments, people, teams)),
                Err(e) => {
                    eprintln!("Error generating assignments: {:?}", e);
                    None
                }
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

        build_assignment_plan(people, teams, flow_assignments).ok() // TODO: better error handling
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
    let on_selection_change = Callback::new(move |((person_name, team, position), is_checked): (
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
    });

    let on_person_hover = Callback::new(
        move |(person, assignment, coords): (Person, Option<String>, (f64, f64))| {
            hovered_person.set(Some((person, assignment)));
            mouse_position.set(coords);
        });

    let on_person_leave = Callback::new(move |_| {
        hovered_person.set(None);
    });

    // Popup handlers
    let on_role_popup_open = Callback::new(
        move |(position, team_name, current_person, coords): (Position, String, Option<Person>, (f64, f64))| {
            role_popup_state.set(Some((position, team_name, current_person, coords)));
        }
    );

    let on_role_popup_close = Callback::new(move |_| {
        role_popup_state.set(None);
    });

    let on_role_swap = Callback::new(move |person_name: String| {
        if let Some((position, team_name, current_person, _)) = role_popup_state() {
            // Implement the simplified approach: unassign current person and assign new person
            app_state.with_mut(|state| {
                // Remove current assignment if someone is assigned
                if let Some(_current) = current_person {
                    state.persistent_locks.remove(&(team_name.clone(), position.clone()));
                }
                
                // Add new assignment (unless empty string for unassign)
                if !person_name.is_empty() {
                    state.persistent_locks.insert((team_name, position), person_name);
                }
            });
        }
    });

    // Handle InteractionBar actions
    let on_interaction_action = move |action: InteractionAction| {
        match action {
            InteractionAction::SetMode(mode) => {
                interaction_mode.set(mode);
                selected_assignments.set(vec![]);
            }
            InteractionAction::ExecuteLock => {
                app_state.with_mut(|state| {
                    execute_lock_action(&selected_assignments(), &mut state.persistent_locks);
                });
                selected_assignments.set(vec![]);
                interaction_mode.set(InteractionMode::ViewOnly);
            }
            InteractionAction::ClearLocks => {
                app_state.with_mut(|state| {
                    state.persistent_locks.clear();
                });
            }
            InteractionAction::SaveState => {
                // Extract current data for export
                let current_raw_data = raw_data.read();
                if let Some((_, ref people, ref teams)) = *current_raw_data {
                    // Extract qual_defs from app state
                    let app_state_read = app_state.read();
                    let qual_defs = app_state_read.files
                        .get("Qual Defs")
                        .and_then(|config| config.parsed_data.as_ref())
                        .and_then(|data| match data {
                            crate::utilities::config::ParsedData::QualDefs(quals) => Some(quals.as_ref()),
                            _ => None,
                        })
                        .cloned()
                        .unwrap_or_default();
                    
                    let save_state = SaveState::new(
                        selected_date(),
                        people,
                        teams,
                        &qual_defs,
                        &app_state_read.persistent_locks,
                    );
                    
                    // Trigger download with timestamp
                    #[cfg(target_arch = "wasm32")]
                    {
                        let timestamp = save_state.export_timestamp.format("%Y%m%d_%H%M%S");
                        let filename = format!("roboamo-save-state-{}.json", timestamp);
                        if let Err(e) = save_state.download(&filename) {
                            web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&format!("Failed to download save state: {}", e)));
                        }
                    }
                    
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // For non-WASM platforms, just log the JSON
                        match save_state.to_json() {
                            Ok(json) => println!("Save State JSON: {}", json),
                            Err(e) => eprintln!("Failed to export save state: {}", e),
                        }
                    }
                }
            }
        }
    };

    use_context_provider(|| ui_context);

    rsx! {
        div {
            class: "results-container",

        // Header with summary stats
        AssignmentStats {
            assignments_signal: assignments,
        }
        // Interaction toolbar
        InteractionBar {
            interaction_mode_signal: interaction_mode,
            selected_count_signal: use_memo(move || selected_assignments().len()),
            persistent_locks_count_signal: use_memo(move || app_state().persistent_locks.len()),
            on_action: on_interaction_action,
        }
        // Analysis date selector
        AnalysisDateBar {
            selected_date_signal: selected_date,
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
                        assignments_signal: assignments,
                        analysis_date_signal: selected_date,
                        on_selection_change: on_selection_change,
                        on_person_hover: on_person_hover,
                        on_person_leave: on_person_leave,
                        on_role_popup_open: on_role_popup_open,
                    }
                }
            }
        }

        // Unassigned Personnel
        if !unassigned_people().is_empty() {
            UnassignedTable {
                assignments_signal: assignments,
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

        // Render popup inside main container like PlayerCard
        if let Some((position, team_name, current_person, popup_position)) = role_popup_state() {
            RolePopup {
                position,
                team_name,
                current_person,
                assignments_signal: assignments,
                popup_position,
                on_swap: on_role_swap,
                on_close: on_role_popup_close,
            }
        }
    } // Close main div
    } // Close rsx! block
}

// Helper functions for interaction actions

fn execute_lock_action(
    selections: &[(String, Option<String>, Option<Position>)],
    persistent_locks: &mut HashMap<(String, Position), String>,
) {
    for (person, team, pos) in selections {
        if let (Some(team), Some(pos)) = (team, pos) {
            persistent_locks.insert((team.clone(), pos.clone()), person.clone());
        }
    }
}

fn should_add_selection(interaction_mode: InteractionMode, current_count: usize) -> bool {
    match interaction_mode {
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
