use dioxus::prelude::*;
use itertools::Itertools;
#[cfg(target_arch = "wasm32")]
use {wasm_bindgen, web_sys};

use crate::engine::{
    assignment::AssignmentPlan,
    person::{DutyStatus, Person},
    team::Position,
};
use crate::views::results::AssignmentUIContext;
use crate::components::InteractionMode;

#[component]
pub fn RolePopup(
    // Core data
    position: Position,
    team_name: String,
    current_person: Option<Person>,
    assignments_signal: ReadOnlySignal<Option<AssignmentPlan>>,
    
    // Positioning
    popup_position: (f64, f64),
    
    // Actions
    on_swap: Callback<String>,
    on_close: Callback<()>,
) -> Element {
    let ui_context = use_context::<AssignmentUIContext>();
    
    // Get people who can fill this role with their current assignments
    let eligible_people = use_memo({
        let position_clone = position.clone();
        let current_person_clone = current_person.clone();
        move || {
            let people = (ui_context.people)();
            let Some(assignments_plan) = assignments_signal() else {
                return Vec::new();
            };
            
            people
                .iter()
                .filter(|person| {
                    // Must have the required qualification
                    person.qualifications.contains(&position_clone.qualification) &&
                    // Don't include the person currently in this role
                    current_person_clone.as_ref().map_or(true, |cp| cp.name != person.name)
                })
                .map(|person| {
                    // Find current assignment for this person
                    let current_assignment = assignments_plan.assignments.iter()
                        .find(|assignment| assignment.person.name == person.name);
                    
                    (person.clone(), current_assignment.cloned())
                })
                .sorted_by_key(|(person, assignment)| {
                    // Sort by: unassigned first, then TAR first, then by PRD date
                    (
                        assignment.is_some(), // unassigned people (None) sort first
                        person.duty_status != DutyStatus::Tar,
                        person.prd.unwrap_or(chrono::NaiveDate::from_ymd_opt(2099, 12, 31).unwrap())
                    )
                })
                .collect::<Vec<_>>()
        }
    });

    // Calculate popup position with edge detection
    let (x, y) = popup_position;

    let eligible_people_list = eligible_people();

    rsx! {
        // Overlay to capture clicks outside popup
        div {
            class: "fixed inset-0",
            style: "z-index: 9998;",
            onclick: move |_| on_close.call(()),
        }
        
        // Main popup - styled like PlayerCard
        div {
            class: "absolute bg-white border border-gray-300 rounded-lg shadow-lg p-4 max-w-sm",
            style: "left: {x + 100.0}px; top: {y - 100.0}px; z-index: 9999; width: 300px;",
            onclick: |e| e.stop_propagation(), // Prevent closing when clicking inside
            
            // Header - slimmed down
            div {
                class: "font-bold text-base mb-1",
                "{position.qualification}"
            }
            if let Some(ref person) = current_person {
                div {
                    class: "text-xs text-blue-600 mb-2",
                    "Current: {person.name} ({person.raterank})"
                }
            } else {
                div {
                    class: "text-xs text-gray-500 mb-2 italic",
                    "Currently unassigned"
                }
            }
            
            // Eligible people section
            if eligible_people_list.is_empty() {
                div {
                    class: "text-xs text-gray-500 mb-3",
                    "No other qualified personnel available"
                }
            } else {
                div {
                    class: "text-xs text-gray-500 mb-2",
                    "Eligible Personnel ({eligible_people_list.len()}):"
                }
                div {
                    class: "space-y-0.5 mb-3 max-h-32 overflow-y-auto",
                    for (person, assignment) in eligible_people_list.iter() {
                        div {
                            key: "{person.name}",
                            class: "flex items-center justify-between p-1.5 rounded hover:bg-gray-50 border border-gray-100",
                            
                            // Person info - streamlined
                            div {
                                class: "flex-1 min-w-0",
                                div {
                                    class: "font-semibold text-xs",
                                    "({person.duty_status.as_str()}) {person.raterank} {person.name.split(',').next().unwrap_or(&person.name)}"
                                    if let Some(prd) = person.prd {
                                        " • {prd.format(\"%m/%y\")}"
                                    }
                                }
                                if let Some(ref assignment) = assignment {
                                    div {
                                        class: "text-xs text-blue-600",
                                        "{assignment.team_name} ({assignment.position.qualification})"
                                    }
                                } else {
                                    div {
                                        class: "text-xs text-gray-400 italic",
                                        "Unassigned"
                                    }
                                }
                            }
                            
                            // Swap button
                            button {
                                class: "ml-2 px-2 py-0.5 bg-blue-600 text-white text-xs rounded hover:bg-blue-700 transition-colors",
                                onclick: {
                                    let person_name = person.name.clone();
                                    move |_| {
                                        on_swap.call(person_name.clone());
                                        on_close.call(());
                                    }
                                },
                                "Swap"
                            }
                        }
                    }
                }
            }
            
            // Footer actions
            div {
                class: "flex justify-between gap-2 pt-2 border-t border-gray-200",
                
                // Leave unassigned option (if someone is currently assigned)
                if current_person.is_some() {
                    button {
                        class: "flex-1 px-3 py-1.5 text-gray-600 text-xs rounded hover:bg-gray-100 transition-colors border border-gray-200",
                        onclick: move |_| {
                            on_swap.call(String::new()); // Empty string = unassign
                            on_close.call(());
                        },
                        "Unassign"
                    }
                }
                
                button {
                    class: "flex-1 px-3 py-1.5 bg-gray-200 text-gray-700 text-xs rounded hover:bg-gray-300 transition-colors",
                    onclick: move |_| on_close.call(()),
                    "Cancel"
                }
            }
        }
    }
}

