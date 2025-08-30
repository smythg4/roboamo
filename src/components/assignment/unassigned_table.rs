use chrono::NaiveDate;
use dioxus::prelude::*;
use itertools::Itertools;

use crate::components::{SearchBar, TeamRow};
use crate::engine::{assignment::AssignmentPlan, person::Person, team::Position};
use crate::views::results::AssignmentUIContext;

#[component]
pub fn UnassignedTable(
    assignments_signal: ReadOnlySignal<AssignmentPlan>,
    analysis_date_signal: Signal<NaiveDate>,
    on_selection_change: EventHandler<((String, Option<String>, Option<Position>), bool)>,
    on_person_hover: EventHandler<(Person, Option<String>, (f64, f64))>,
    on_person_leave: EventHandler<()>,
) -> Element {
    // Get context for shared UI state
    let ui_context = use_context::<AssignmentUIContext>();
    // Local search query state
    let mut search_query = use_signal(|| String::new());

    // Read current value from signal
    let analysis_date = analysis_date_signal();

    // Memoize unassigned people extraction and sorting - only recalculates when assignments change
    let unassigned_people = use_memo(move || {
        let assignments = assignments_signal();
        assignments
            .unassigned_people
            .iter()
            .sorted_by(|p, q| Ord::cmp(&q.qualifications.len(), &p.qualifications.len()))
            .cloned()
            .collect::<Vec<_>>()
    });

    // Memoize filtered people - only recalculates when search query or unassigned people change
    let filtered_people = use_memo(move || {
        let query = search_query().to_lowercase();
        let people = unassigned_people();

        people
            .iter()
            .filter(|p| {
                query.is_empty()
                    || p.name.to_lowercase().contains(&query)
                    || p.qualifications
                        .iter()
                        .any(|q| q.to_lowercase().contains(&query))
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    rsx! {
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
                            th { class: "table-header-cell", "Eligible Roles" }
                            th { class: "table-header-cell", "Name" }
                            th { class: "table-header-cell", "Rate/Rank" }
                            th { class: "table-header-cell", "Status" }
                            th { class: "table-header-cell", "PRD" }

                        }
                    }
                    tbody {
                        for person in filtered_people().iter() {
                            TeamRow {
                                assignment: None, // Unassigned people have no assignment
                                person: person.clone(),
                                team: None, // Unassigned people have no team
                                analysis_date: analysis_date,
                                on_selection_change: on_selection_change,
                                on_person_hover: on_person_hover,
                                on_person_leave: on_person_leave,
                            }
                        }
                    }
                }
            }
        }
    }
}
