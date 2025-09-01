use dioxus::prelude::*;

use crate::engine::{assignment::AssignmentPlan, person::DutyStatus};

#[component]
pub fn AssignmentStats(assignments_signal: ReadOnlySignal<Option<AssignmentPlan>>) -> Element {
    // Memoize expensive stats calculations - only recalculates when assignments change
    let stats = use_memo(move || {
        let Some(assignments) = assignments_signal() else {
            return (0, 0, 0, 0, 0, 100);
        };

        let assigned_selres_count = assignments
            .assignments
            .iter()
            .filter(|assignment| assignment.person.duty_status == DutyStatus::Selres)
            .count();

        let assigned_aw_count = assignments
            .assignments
            .iter()
            .filter(|assignment| assignment.person.raterank.starts_with("AW"))
            .count();

        let total_positions = assignments.assignments.len() + assignments.unfilled_positions.len();
        let fill_percentage = if total_positions > 0 {
            (assignments.assignments.len() as f32 / total_positions as f32 * 100.0).round() as u32
        } else {
            100
        };

        (
            assignments.assignments.len(),
            assignments.unassigned_people.len(),
            assignments.unfilled_positions.len(),
            assigned_selres_count,
            assigned_aw_count,
            fill_percentage,
        )
    });

    let (total_assigned, total_unassigned, total_unfilled, total_selres_used, total_aw_used, fill_percentage) =
        stats();

    rsx! {
        div {
            class: "results-header m-1",
            h1 {
                class: "results-title",
                "Assignment Results"
            }
            div {
                class: "stats-grid",
                div {
                    class: "stat-card-assigned",
                    h3 { class: "stat-number-green", "{total_assigned}" }
                    p { class: "stat-label-green", "People Assigned ({fill_percentage}%)" }
                }
                div {
                    class: "stat-card-unfilled",
                    h3 { class: "stat-number-red", "{total_unfilled}" }
                    p { class: "stat-label-red", "Unfilled Positions" }
                }
                div {
                    class: "stat-card-selres",
                    h3 { class: "stat-number-yellow", "{total_selres_used}" }
                    p { class: "stat-label-yellow", "SELRES Used" }
                }
                div {
                    class: "stat-card-aw",
                    h3 { class: "stat-number-purple", "{total_aw_used}" }
                    p { class: "stat-label-purple", "AWs Used" }
                }
                div {
                    class: "stat-card-unassigned",
                    h3 { class: "stat-number-blue", "{total_unassigned}" }
                    p { class: "stat-label-blue", "Available Personnel" }
                }
            }
        }
    }
}
