use dioxus::prelude::*;
use crate::Route;
use crate::utilities::AppState;

#[component]
pub fn ProgressBar() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let (up, tot) = app_state().upload_progress();
    let nav = use_navigator();
    let percentage = ((up as f32 / tot as f32) * 100.0) as u32;
    
    rsx! {
        div {
            class: "flex items-center gap-4",
            
            // Progress indicator
            div {
                class: "flex items-center gap-3",
                
                // Progress ring
                div {
                    class: "relative",
                    svg {
                        class: "w-10 h-10 transform -rotate-90",
                        circle {
                            cx: "20",
                            cy: "20",
                            r: "16",
                            stroke: "currentColor",
                            "stroke-width": "4",
                            fill: "none",
                            class: "text-gray-200"
                        }
                        circle {
                            cx: "20",
                            cy: "20",
                            r: "16",
                            stroke: "currentColor",
                            "stroke-width": "4",
                            fill: "none",
                            class: "text-blue-600",
                            "stroke-dasharray": format!("{} {}", percentage, 100 - percentage),
                            "stroke-dashoffset": "25",
                            style: "transition: stroke-dasharray 0.3s ease"
                        }
                    }
                    span {
                        class: "absolute inset-0 flex items-center justify-center text-xs font-bold text-gray-700",
                        "{up}/{tot}"
                    }
                }
                
                // Text indicator
                div {
                    class: "flex flex-col",
                    span {
                        class: "text-xs font-medium text-gray-500",
                        "Progress"
                    }
                    span {
                        class: "text-sm font-bold text-gray-900",
                        if up == tot {
                            "Complete!"
                        } else {
                            "{percentage}%"
                        }
                    }
                }
            }
            
            // Reset button
            button {
                onclick: move |_| {
                    app_state.set(AppState::default());
                    nav.push(Route::Home {});
                },
                class: "flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-gray-600 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 hover:text-red-600 hover:border-red-300 transition-all duration-200 group",
                svg {
                    class: "w-4 h-4 group-hover:rotate-180 transition-transform duration-300",
                    fill: "none",
                    stroke: "currentColor",
                    "stroke-width": "2",
                    "viewBox": "0 0 24 24",
                    path {
                        "stroke-linecap": "round",
                        "stroke-linejoin": "round",
                        d: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                    }
                }
                "Reset"
            }
        }
    }
}