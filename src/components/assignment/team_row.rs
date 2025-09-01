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
use crate::views::results::{
    AssignmentUIContext, PersonHoverHandler, PersonLeaveHandler, RolePopupOpenHandler,
    SelectionChangeHandler,
};

#[component]
pub fn TeamRow(
    assignment: Option<Assignment>, // None for unassigned people
    person: Person,                 // Always present
    team: Option<Team>,             // None for unassigned people
    analysis_date: NaiveDate,
    on_selection_change: SelectionChangeHandler,
    on_person_hover: PersonHoverHandler,
    on_person_leave: PersonLeaveHandler,
    on_role_popup_open: RolePopupOpenHandler,
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
            false // Swap eligibility removed with old checkbox swap system
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
                class: "table-cell-checkbox",
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
                class: "table-cell-compact",
                if let Some(position) = &position_opt {
                    RoleBadge {
                        qualification: position.qualification.clone(),
                        is_locked: is_manual_override,
                        on_click: if team_name_opt.is_some() {
                            Some({
                                let position = position.clone();
                                let team_name = team_name_opt.clone().unwrap();
                                let current_person = Some(person.clone());
                                Callback::new(move |(x, y): (f64, f64)| {
                                    on_role_popup_open.call((position.clone(), team_name.clone(), current_person.clone(), (x, y)));
                                })
                            })
                        } else {
                            None
                        },
                    }
                } else {
                    // For unassigned people, show their qualifications (no popup needed)
                    RoleBadge {
                        qualification: "{person.qualifications.iter().sorted().join(\", \")}",
                        is_locked: is_manual_override,
                        on_click: None, // No popup for unassigned people
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
                class: "table-cell-muted-compact",
                "{person.raterank}"
            }

            // Status cell
            td {
                class: "table-cell-compact",
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
                class: "table-cell-muted-compact",
                if let Some(prd) = person.prd {
                    span {
                        class: get_prd_css_class(prd, analysis_date),
                        "{prd.format(\"%m/%y\")}"
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
        InteractionMode::Lock => false,
        InteractionMode::ViewOnly => true,
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
