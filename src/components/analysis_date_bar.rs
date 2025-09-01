use chrono::NaiveDate;
use dioxus::prelude::*;

#[component]
pub fn AnalysisDateBar(selected_date_signal: Signal<NaiveDate>) -> Element {
    let selected_date = selected_date_signal();

    rsx! {
        div {
            class: "sticky top-30 z-50 bg-white shadow-md border border-gray-200 rounded-lg flex items-center gap-1 p-2 m-1 w-auto",
            label {
                class: "text-xs font-medium text-gray-700",
                span { "📅 " }
                "Analysis Date:"
            }
            input {
                r#type: "date",
                class: "border border-gray-300 rounded px-2 py-1 text-sm",
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
