// Standard library imports
use std::collections::HashMap;
use std::rc::Rc;

// External crate imports
use dioxus::prelude::*;
use itertools::Itertools;

// Local crate imports - engine
use crate::engine::{
    assignment::{AssignmentLock, FlowAssignment},
    builder::{build_assignment_plan, generate_assignments, AssignmentResult},
    person::Person,
    team::{Position, Team},
};

// Type aliases to reduce complexity in function signatures
pub type SelectionChangeHandler = Callback<((String, Option<String>, Option<Position>), bool)>;
pub type PersonHoverHandler = Callback<(Person, Option<String>, (f64, f64))>;
pub type PersonLeaveHandler = Callback<()>;
pub type AssignmentSelection = Vec<(String, Option<String>, Option<Position>)>;

// Local crate imports - other
use crate::{
    components::{AnalysisDateBar, AssignmentStats, InteractionAction, InteractionBar, InteractionMode, PlayerCard, TeamCard, UnassignedTable},
    utilities::AppState,
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

    // Handle InteractionBar actions
    let on_interaction_action = move |action: InteractionAction| {
        match action {
            InteractionAction::SetMode(mode) => {
                interaction_mode.set(mode);
                selected_assignments.set(vec![]);
            }
            InteractionAction::ExecuteSwap => {
                execute_swap_action(&selected_assignments(), &mut persistent_locks);
                selected_assignments.set(vec![]);
                interaction_mode.set(InteractionMode::ViewOnly);
            }
            InteractionAction::ExecuteLock => {
                execute_lock_action(&selected_assignments(), &mut persistent_locks);
                selected_assignments.set(vec![]);
                interaction_mode.set(InteractionMode::ViewOnly);
            }
            InteractionAction::ClearLocks => {
                persistent_locks.set(HashMap::new());
            }
        }
    };

    use_context_provider(|| ui_context);

    rsx! {
        div {
            class: "results-container",

        // Header with summary stats
        AssignmentStats {
            assignments_signal: assignments.read().clone().unwrap(),
        }
        // Interaction toolbar
        InteractionBar {
            interaction_mode_signal: interaction_mode,
            selected_count_signal: use_memo(move || selected_assignments().len()),
            persistent_locks_count_signal: use_memo(move || persistent_locks().len()),
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

// Helper functions for interaction actions
fn execute_swap_action(
    selections: &[(String, Option<String>, Option<Position>)],
    persistent_locks: &mut Signal<HashMap<(String, Position), String>>,
) {
    if selections.len() == 2 {
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
    }
}

fn execute_lock_action(
    selections: &[(String, Option<String>, Option<Position>)],
    persistent_locks: &mut Signal<HashMap<(String, Position), String>>,
) {
    persistent_locks.with_mut(|locks| {
        for (person, team, pos) in selections {
            if let (Some(team), Some(pos)) = (team, pos) {
                locks.insert((team.clone(), pos.clone()), person.clone());
            }
        }
    });
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
