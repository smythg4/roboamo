use chrono::NaiveDate;
use dioxus::prelude::*;
use itertools::Itertools;
use std::rc::Rc;

use crate::components::{InteractionMode, TeamRow};
use crate::engine::{
    assignment::{Assignment, AssignmentPlan},
    person::Person,
    team::{Position, Team},
};
use crate::views::results::AssignmentUIContext;

#[component]
pub fn TeamCard(
    team: Team,
    assignments_signal: ReadOnlySignal<AssignmentPlan>,
    analysis_date_signal: Signal<NaiveDate>,
    on_selection_change: EventHandler<((String, Option<String>, Option<Position>), bool)>,
    on_person_hover: EventHandler<(Person, Option<String>, (f64, f64))>,
    on_person_leave: EventHandler<()>,
) -> Element {
    // Note: Context available but not used in this component yet
    let _ui_context = use_context::<AssignmentUIContext>();
    // Memoize team assignments - only recalculates when assignments change
    let team_assignments = use_memo({
        let team_name = team.name.clone();
        move || {
            let assignments = assignments_signal();
            assignments
                .assignments
                .iter()
                .filter(|a| a.team_name == team_name)
                .cloned()
                .sorted_by_key(|a| !a.manual_override)
                .collect::<Vec<_>>()
        }
    });

    // Memoize unfilled positions for this team - only recalculates when assignments change
    let unfilled_positions = use_memo({
        let team_name = team.name.clone();
        move || {
            let assignments = assignments_signal();
            assignments
                .unfilled_positions
                .iter()
                .filter(|(tn, _)| tn == &team_name)
                .map(|(_, role_id)| role_id.clone())
                .collect::<Vec<_>>()
        }
    });

    let team_assignments_vec = team_assignments();
    let unfilled_positions_vec = unfilled_positions();
    rsx! {
        div {
            class: "team-card",
            h3 {
                class: "team-header",
                span { class: "team-icon", "👥" }
                "{team.name} ({team_assignments_vec.len()} assigned)"
            }

            div {
                class: "table-wrapper",
                table {
                    class: "results-table",
                    thead {
                        class: "table-header",
                        tr {
                            th { class: "table-header-cell", "" }
                            th { class: "table-header-cell", "Role" }
                            th { class: "table-header-cell", "Name" }
                            th { class: "table-header-cell", "Rate/Rank" }
                            th { class: "table-header-cell", "Status" }
                            th { class: "table-header-cell", "PRD" }
                        }
                    }
                    tbody {
                        for assignment in team_assignments_vec {
                            TeamRow {
                                assignment: Some(assignment.clone()),
                                person: assignment.person.as_ref().clone(),
                                team: Some(team.clone()),
                                analysis_date: analysis_date_signal(),
                                on_selection_change: on_selection_change,
                                on_person_hover: on_person_hover,
                                on_person_leave: on_person_leave,
                            }
                        }

                    // rows for missing quals
                    for missing_role_id in unfilled_positions_vec {
                        tr {
                            class: "table-row bg-red-50",
                            td {
                                ""
                            }
                            td {
                                class: "table-cell-name text-red-600",
                                span { class: "text-xl mr-2", "⚠️" }
                            }
                            td { class: "table-cell-muted text-red-400", "" }
                            td {
                                class: "table-cell",
                                span {
                                    class: "role-badge bg-red-100 text-red-800",
                                    "{missing_role_id}"
                                }
                            }
                            td { class: "table-cell-muted text-red-400", "" }
                            td { class: "table-cell-muted text-red-400", "" }
                        }
                    }
                } // Close tbody
            } // Close table
        } // Close div (table-wrapper)
    } // Close div (team-card)
    } // Close rsx!
}

// Helper functions moved to TeamRow component
