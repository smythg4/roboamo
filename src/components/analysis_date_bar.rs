use chrono::NaiveDate;
use dioxus::prelude::*;

#[component]
pub fn AnalysisDateBar(
    selected_date_signal: Signal<NaiveDate>,
) -> Element {
    let selected_date = selected_date_signal();
    
    rsx! {
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
                value: "{selected_date.format(\"%Y-%m-%d\")}",
                onchange: move |evt| {
                    if let Ok(new_date) = chrono::NaiveDate::parse_from_str(&evt.value(), "%Y-%m-%d") {
                        selected_date_signal.set(new_date);
                    }
                }
            }
        }
    }
}