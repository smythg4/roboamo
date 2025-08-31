use chrono::{Datelike, NaiveDate};
use dioxus::html::MouseData;
use dioxus::prelude::*;
use itertools::Itertools;

use crate::components::{InteractionMode, RoleBadge};
use crate::engine::{
    assignment::Assignment,
    person::{DutyStatus, Person},
    team::{Position, Team},
};
use crate::views::results::{AssignmentUIContext, SelectionChangeHandler, PersonHoverHandler, PersonLeaveHandler};

#[component]
pub fn TeamRow(
    assignment: Option<Assignment>, // None for unassigned people
    person: Person,                 // Always present
    team: Option<Team>,             // None for unassigned people
    analysis_date: NaiveDate,
    on_selection_change: SelectionChangeHandler,
    on_person_hover: PersonHoverHandler,
    on_person_leave: PersonLeaveHandler,
) -> Element {
    // Get context for shared UI state
    let ui_context = use_context::<AssignmentUIContext>();

    // Extract assignment data or use None for unassigned
    let (team_name_opt, position_opt, is_manual_override) = match &assignment {
        Some(assignment) => (
            Some(assignment.team_name.clone()),
            Some(assignment.position.clone()),
            assignment.manual_override,
        ),
        None => (None, None, false),
    };

    // Memoize selection state - only recalculates when selections change
    let is_selected = use_memo({
        let person_name = person.name.clone();
        let team_name_opt = team_name_opt.clone();
        let position_opt = position_opt.clone();
        let ui_context = ui_context.clone();
        move || {
            let selections = (ui_context.selected_assignments)();
            match (&team_name_opt, &position_opt) {
                (Some(team_name), Some(position)) => {
                    is_assignment_selected(&selections, &person_name, team_name, position)
                }
                (None, None) => is_unassigned_selected(&selections, &person_name),
                _ => false, // Invalid state
            }
        }
    });

    // Memoize swap eligibility - only recalculates when mode/selections/people change
    let is_eligible = use_memo({
        let person_name = person.name.clone();
        let team_name_opt = team_name_opt.clone();
        let position_opt = position_opt.clone();
        let ui_context = ui_context.clone();
        move || {
            let interaction_mode = (ui_context.interaction_mode)();
            let selected_assignments = (ui_context.selected_assignments)();
            let people = (ui_context.people)();
            is_swap_eligible(
                interaction_mode,
                &selected_assignments,
                &person_name,
                team_name_opt.as_deref(),
                position_opt.as_ref(),
                &people,
            )
        }
    });

    // Memoize checkbox disabled state - only recalculates when mode/selections change
    let is_disabled = use_memo({
        let is_selected = is_selected();
        let ui_context = ui_context.clone();
        move || {
            let interaction_mode = (ui_context.interaction_mode)();
            let selected_count = (ui_context.selected_assignments)().len();
            is_checkbox_disabled(interaction_mode, selected_count, is_selected)
        }
    });

    // Create separate clones for closures to avoid ownership conflicts
    let person_hover = person.clone();
    let team_name_hover = team_name_opt.clone();
    let qualification_hover = position_opt.as_ref().map(|p| p.qualification.clone());

    rsx! {
        tr {
            class: format!(
                "{}",
                if is_selected() {
                    "table-row bg-yellow-50 border-l-4 border-l-yellow-400"
                } else if is_eligible() {
                    "table-row bg-emerald-100 border-l-4 border-l-emerald-400"
                } else if is_manual_override {
                    "table-row border-black border bg-gray-200"
                } else {
                    "table-row"
                }
            ),

            // Checkbox cell
            td {
                class: "table-cell-muted",
                input {
                    r#type: "checkbox",
                    style: if (ui_context.interaction_mode)() == InteractionMode::ViewOnly {
                        "visibility: hidden;"
                    } else {
                        "visibility: visible;"
                    },
                    checked: is_selected(),
                    disabled: is_disabled(),
                    onchange: move |evt| {
                        let assignment_id = (
                            person.name.clone(),
                            team_name_opt.clone(),
                            position_opt.clone(),
                        );
                        on_selection_change.call((assignment_id, evt.checked()));
                    }
                }
            }

            // Role badge cell (or qualifications for unassigned)
            td {
                class: "table-cell",
                if let Some(position) = &position_opt {
                    RoleBadge {
                        qualification: position.qualification.clone(),
                        is_locked: is_manual_override,
                    }
                } else {
                    // For unassigned people, show their qualifications
                    RoleBadge {
                        qualification: "{person.qualifications.iter().sorted().join(\", \")}",
                        is_locked: is_manual_override,
                    }
                }
            }

            // Name cell with hover
            td {
                class: "table-cell-name",
                onmouseenter: move |evt: Event<MouseData>| {
                    let current_assignment = match (&team_name_hover, &qualification_hover) {
                        (Some(team), Some(qual)) => Some(format!("{} - {}", team, qual)),
                        _ => None, // Unassigned
                    };
                    let coords = evt.data().client_coordinates();
                    on_person_hover.call((person_hover.clone(), current_assignment, (coords.x, coords.y)));
                },
                onmouseleave: move |_| {
                    on_person_leave.call(());
                },
                "{person.name}"
            }

            // Rate/Rank cell
            td {
                class: "table-cell-muted",
                "{person.raterank}"
            }

            // Status cell
            td {
                class: "table-cell",
                match person.duty_status {
                    DutyStatus::Tar => rsx! {
                        span {
                            class: "status-badge-tar",
                            "TAR"
                        }
                    },
                    DutyStatus::Selres => rsx! {
                        span {
                            class: "status-badge-selres",
                            "SELRES"
                        }
                    }
                }
            }

            // PRD cell
            td {
                class: "table-cell-muted",
                if let Some(prd) = person.prd {
                    span {
                        class: get_prd_css_class(prd, analysis_date),
                        "{prd}"
                    }
                } else {
                    "-"
                }
            }
        }
    }
}

// Helper functions
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

fn is_unassigned_selected(
    selections: &[(String, Option<String>, Option<Position>)],
    person_name: &str,
) -> bool {
    selections
        .iter()
        .any(|(name, team, pos)| name == person_name && team.is_none() && pos.is_none())
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

    // Find both people in the people list
    let Some(selected_person) = people.iter().find(|p| &p.name == selected_person_name) else {
        return false;
    };
    let Some(target_person) = people.iter().find(|p| p.name == assignment_person) else {
        return false;
    };

    match (selected_position, assignment_position) {
        // Selected person is unassigned, target person is assigned
        (None, Some(assignment_position)) => {
            // Highlight if the unassigned person can do the target's job
            selected_person
                .qualifications
                .contains(&assignment_position.qualification)
        }
        // Selected person is assigned, target person is unassigned
        (Some(selected_position), None) => {
            // Highlight if the unassigned target can do the selected person's job
            target_person
                .qualifications
                .contains(&selected_position.qualification)
        }
        // Both people are assigned - mutual qualification required
        (Some(selected_position), Some(assignment_position)) => {
            let can_selected_do_target = selected_person
                .qualifications
                .contains(&assignment_position.qualification);
            let can_target_do_selected = target_person
                .qualifications
                .contains(&selected_position.qualification);

            can_selected_do_target && can_target_do_selected
        }
        // Both people are unassigned - no highlighting needed
        (None, None) => false,
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
