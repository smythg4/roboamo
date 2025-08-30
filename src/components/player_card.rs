use crate::engine::person::Person;
use dioxus::prelude::*;

#[component]
pub fn PlayerCard(
    person: Person,
    current_assignment: Option<String>,
    position: (f64, f64),
) -> Element {
    rsx! {
        div {
            class: "fixed z-50 bg-white border border-gray-300 rounded-lg shadow-lg p-4 max-w-sm pointer-events-none",
            style: "left: {position.0 + 10.0}px; top: {position.1 + 10.0}px;",

            div {
                class: "font-bold text-lg mb-2",
                "{person.name}"
            }
            div {
                class: "text-sm text-gray-600 mb-2",
                "Rate/Rank: {person.raterank}"
            }
            if let Some(assignment) = current_assignment {
                div {
                    class: "text-sm text-blue-600 mb-2",
                    "Current: {assignment}"
                }
            }
            div {
                class: "text-xs text-gray-500",
                "Qualifications: {person.qualifications.iter().cloned().collect::<Vec<_>>().join(\", \")}"
            }
        }
    }
}
