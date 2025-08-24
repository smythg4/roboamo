// src/components/error_display.rs
use dioxus::prelude::*;

#[component]
pub fn ErrorDisplay(error: String, retry: Option<EventHandler>) -> Element {
    rsx! {
        div {
            class: "min-h-[400px] flex items-center justify-center p-8",
            div {
                class: "max-w-md w-full bg-red-50 border border-red-200 rounded-lg p-6",
                div {
                    class: "flex items-start",
                    div {
                        class: "flex-shrink-0",
                        svg {
                            class: "h-6 w-6 text-red-600",
                            fill: "none",
                            stroke: "currentColor",
                            "viewBox": "0 0 24 24",
                            path {
                                "stroke-linecap": "round",
                                "stroke-linejoin": "round",
                                "stroke-width": "2",
                                d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                            }
                        }
                    }
                    div {
                        class: "ml-3 flex-1",
                        h3 {
                            class: "text-sm font-medium text-red-800",
                            "Something went wrong"
                        }
                        div {
                            class: "mt-2 text-sm text-red-700",
                            p { "{error}" }
                        }
                        if let Some(retry_handler) = retry {
                            button {
                                onclick: move |_| retry_handler.call(()),
                                class: "mt-4 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors text-sm font-medium",
                                "Try Again"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn LoadingSpinner() -> Element {
    rsx! {
        div {
            class: "flex items-center justify-center p-8",
            div {
                class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"
            }
        }
    }
}