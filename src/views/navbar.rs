use crate::Route;
use dioxus::prelude::*;

use crate::components::ProgressBar;
use crate::utilities::config::PAGES;
use crate::utilities::AppState;

#[component]
pub fn Navbar() -> Element {
    let state = use_context::<Signal<AppState>>();
    let (n, _) = state().upload_progress();
    let all_complete = state().all_files_uploaded();
    let mut show_mobile_menu = use_signal(|| false);

    rsx! {
        div {
            class: "sticky top-0 z-50 bg-white/95 backdrop-blur-md border-b border-gray-200 shadow-sm",

            // Main navbar container
            div {
                class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div {
                    class: "flex justify-between items-center h-16",

                    // Left side - Logo and primary nav
                    div {
                        class: "flex items-center",

                        // Mobile menu button
                        button {
                            onclick: move |_| show_mobile_menu.set(!show_mobile_menu()),
                            class: "lg:hidden p-2 rounded-md text-gray-600 hover:text-gray-900 hover:bg-gray-100",
                            svg {
                                class: "w-6 h-6",
                                fill: "none",
                                stroke: "currentColor",
                                "viewBox": "0 0 24 24",
                                if show_mobile_menu() {
                                    path {
                                        "stroke-linecap": "round",
                                        "stroke-linejoin": "round",
                                        "stroke-width": "2",
                                        d: "M6 18L18 6M6 6l12 12"
                                    }
                                } else {
                                    path {
                                        "stroke-linecap": "round",
                                        "stroke-linejoin": "round",
                                        "stroke-width": "2",
                                        d: "M4 6h16M4 12h16M4 18h16"
                                    }
                                }
                            }
                        }

                        // Desktop Navigation
                        div {
                            class: "hidden lg:flex items-center space-x-1",

                            // Home link - Compact
                            Link {
                                to: Route::Home {},
                                class: "flex items-center px-2 py-1.5 rounded-lg text-gray-700 hover:bg-gray-100 hover:text-blue-600 transition-all duration-200 font-medium text-sm",
                                svg {
                                    class: "w-4 h-4 mr-1.5",
                                    fill: "none",
                                    stroke: "currentColor",
                                    "stroke-width": "2",
                                    "viewBox": "0 0 24 24",
                                    path {
                                        "stroke-linecap": "round",
                                        "stroke-linejoin": "round",
                                        d: "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"
                                    }
                                }
                                "Home"
                            }

                            // Roadmap link - Compact
                            Link {
                                to: Route::ProductRoadmap {},
                                class: "flex items-center px-2 py-1.5 rounded-lg text-gray-700 hover:bg-gray-100 hover:text-blue-600 transition-all duration-200 font-medium text-sm",
                                svg {
                                    class: "w-4 h-4 mr-1.5",
                                    fill: "none",
                                    stroke: "currentColor",
                                    "stroke-width": "2",
                                    "viewBox": "0 0 24 24",
                                    // Roadmap/timeline icon
                                    path {
                                        "stroke-linecap": "round",
                                        "stroke-linejoin": "round",
                                        d: "M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01"
                                    }
                                }
                                "Roadmap"
                            }

                            // Divider
                            div {
                                class: "h-5 w-px bg-gray-300 mx-1"
                            }

                            // Group all file uploads in a dropdown
                            div {
                                class: "relative group",
                                button {
                                    class: "flex items-center px-3 py-2 rounded-lg text-gray-700 hover:bg-gray-100 hover:text-blue-600 transition-all duration-200 font-medium text-sm",
                                    "Files ({n}/4)"
                                    svg {
                                        class: "w-4 h-4 ml-1",
                                        fill: "none",
                                        stroke: "currentColor",
                                        "viewBox": "0 0 24 24",
                                        path {
                                            "stroke-linecap": "round",
                                            "stroke-linejoin": "round",
                                            "stroke-width": "2",
                                            d: "M19 9l-7 7-7-7"
                                        }
                                    }
                                }
                                div {
                                    class: "absolute hidden group-hover:block top-full left-0 mt-1 w-48 bg-white rounded-lg shadow-lg border border-gray-200 py-1 z-50",
                                    // List of file upload links
                                    for (idx, page) in PAGES.iter().enumerate() {
                                        if idx <= n || idx == 0 {
                                            Link {
                                                to: Route::FileUpload { page: page.to_string() },
                                                class: "flex items-center px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 hover:text-blue-600 transition-colors",
                                                span {
                                                    class: "inline-flex items-center justify-center w-5 h-5 mr-2 text-xs font-bold rounded-full bg-gray-200 text-gray-600",
                                                    "{idx + 1}"
                                                }
                                                "{page}"
                                                if idx < n && idx < PAGES.len() - 1 {
                                                    span {
                                                        class: "ml-auto text-green-500 text-xs",
                                                        "✓"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }


                            // Generate button - Compact version
                            if all_complete {
                                Link {
                                    to: Route::Results {},
                                    class: "ml-2 flex items-center px-3 py-1.5 bg-gradient-to-r from-green-700 to-green-400 text-white rounded-lg font-semibold shadow hover:shadow-lg hover:from-green-600 hover:to-green-300 transform hover:-translate-y-0.5 transition-all duration-200 text-sm",
                                    svg {
                                        class: "w-4 h-4 mr-1.5",
                                        fill: "none",
                                        stroke: "currentColor",
                                        "stroke-width": "2",
                                        "viewBox": "0 0 24 24",
                                        path {
                                            "stroke-linecap": "round",
                                            "stroke-linejoin": "round",
                                            d: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                                        }
                                    }
                                    span {
                                        class: "hidden xl:inline",
                                        "Generate Assignments"
                                    }
                                    span {
                                        class: "xl:hidden",
                                        "See Results"
                                    }
                                }
                            }
                        }
                    }

                    // Right side - Progress (more compact)
                    div {
                        class: "hidden sm:block",
                        ProgressBar { }
                    }
                }
            }

            // Mobile menu dropdown
            if show_mobile_menu() {
                div {
                    class: "lg:hidden border-t border-gray-200 bg-white",
                    div {
                        class: "px-2 pt-2 pb-3 space-y-1",

                        Link {
                            to: Route::Home {},
                            onclick: move |_| show_mobile_menu.set(false),
                            class: "block px-3 py-2 rounded-md text-base font-medium text-gray-700 hover:bg-gray-100",
                            "🏠 Home"
                        }

                        Link {
                            to: Route::ProductRoadmap {},
                            onclick: move |_| show_mobile_menu.set(false),
                            class: "block px-3 py-2 rounded-md text-base font-medium text-gray-700 hover:bg-gray-100",
                            "🚧 Roadmap"
                        }

                        div {
                            class: "border-t border-gray-200 my-2"
                        }

                        for (idx, page) in PAGES.iter().enumerate() {
                            if idx < n || idx == 0 {
                                Link {
                                    to: Route::FileUpload { page: page.to_string() },
                                    onclick: move |_| show_mobile_menu.set(false),
                                    class: "block px-3 py-2 rounded-md text-base font-medium text-gray-700 hover:bg-gray-100",
                                    span {
                                        class: "inline-flex items-center justify-center w-6 h-6 mr-2 text-xs font-bold rounded-full bg-gray-200 text-gray-600",
                                        "{idx + 1}"
                                    }
                                    "{page}"
                                    if idx < n && idx < PAGES.len() - 1 {
                                        span {
                                            class: "ml-2 text-green-500",
                                            "✓"
                                        }
                                    }
                                }
                            }
                        }

                        if all_complete {
                            div {
                                class: "border-t border-gray-200 my-2"
                            }
                            Link {
                                to: Route::Results {},
                                onclick: move |_| show_mobile_menu.set(false),
                                class: "block px-3 py-2 rounded-md text-base font-medium bg-green-600 text-white hover:bg-green-700",
                                "Generate Assignments ✓"
                            }
                        }

                        // Mobile progress
                        div {
                            class: "border-t border-gray-200 mt-2 pt-2",
                            ProgressBar { }
                        }
                    }
                }
            }
        }

        // Main content area
        div {
            class: "min-h-screen bg-gradient-to-br from-gray-50 to-gray-100",
            Outlet::<Route> {}
        }
    }
}
