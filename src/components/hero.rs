use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            class: "hero-container min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50",

            // Main Hero Section
            div {
                class: "hero-header text-center py-12 md:py-20 px-4 sm:px-6 lg:px-8",
                div {
                    class: "hero-badge inline-block px-3 py-1.5 md:px-4 md:py-2 bg-blue-100 text-blue-700 rounded-full text-xs md:text-sm font-semibold mb-4 animate-fade-in",
                    "Readiness Assessment Made Easy"
                }
                div {
                    class: "text-4xl md:text-5xl lg:text-6xl mb-4 animate-bounce-slow",
                    "🤖🔧✈️"
                }
                h1 {
                    class: "hero-title text-5xl md:text-6xl lg:text-7xl font-black bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent mb-4 animate-slide-up",
                    "RoboAMO"
                }
                p {
                    class: "hero-subtitle text-lg md:text-xl lg:text-2xl text-gray-600 max-w-3xl mx-auto mb-8 px-4 animate-slide-up-delay",
                    "Intelligent manpower analysis for Naval Aviation Maintenance"
                }

                div {
                    class: "hero-cta-group flex flex-col sm:flex-row gap-4 justify-center items-center px-4",
                    Link {
                        to: Route::FileUpload { page: "Requirements".to_string() },
                        class: "w-full sm:w-auto px-6 md:px-8 py-3 md:py-4 bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-semibold rounded-xl shadow-lg hover:shadow-xl transform hover:-translate-y-0.5 transition-all duration-200",
                        "Get Started →"
                    }
                    a {
                        href: "#how-it-works",
                        class: "w-full sm:w-auto px-6 md:px-8 py-3 md:py-4 bg-white text-gray-700 font-semibold rounded-xl border-2 border-gray-200 shadow-md hover:shadow-lg transform hover:-translate-y-0.5 transition-all duration-200",
                        "Learn More"
                    }
                }
            }

            // Features Grid
            div {
                class: "features-grid grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 -mt-10 relative z-10",
                div {
                    class: "feature-card bg-white rounded-2xl p-6 md:p-8 shadow-xl hover:shadow-2xl transition-shadow duration-300 text-center",
                    div { class: "feature-icon text-4xl md:text-5xl mb-4", "📊" }
                    h3 { class: "feature-title text-lg md:text-xl font-bold text-gray-900 mb-3", "Smart Analysis" }
                    p { class: "feature-desc text-sm md:text-base text-gray-600 leading-relaxed",
                        "Instantly identify qualification gaps and manning priorities"
                    }
                }
                div {
                    class: "feature-card bg-white rounded-2xl p-6 md:p-8 shadow-xl hover:shadow-2xl transition-shadow duration-300 text-center",
                    div { class: "feature-icon text-4xl md:text-5xl mb-4", "⚡" }
                    h3 { class: "feature-title text-lg md:text-xl font-bold text-gray-900 mb-3", "Rapid Processing" }
                    p { class: "feature-desc text-sm md:text-base text-gray-600 leading-relaxed",
                        "Upload reports and receive insights in under a second"
                    }
                }
                div {
                    class: "feature-card bg-white rounded-2xl p-6 md:p-8 shadow-xl hover:shadow-2xl transition-shadow duration-300 text-center",
                    div { class: "feature-icon text-4xl md:text-5xl mb-4", "🎯" }
                    h3 { class: "feature-title text-lg md:text-xl font-bold text-gray-900 mb-3", "Optimized Assignments" }
                    p { class: "feature-desc text-sm md:text-base text-gray-600 leading-relaxed",
                        "Advanced algorithm for effective personnel utilization"
                    }
                }
                div {
                    class: "feature-card bg-white rounded-2xl p-6 md:p-8 shadow-xl hover:shadow-2xl transition-shadow duration-300 text-center",
                    div { class: "feature-icon text-4xl md:text-5xl mb-4", "🔒" }
                    h3 { class: "feature-title text-lg md:text-xl font-bold text-gray-900 mb-3", "100% Private" }
                    p { class: "feature-desc text-sm md:text-base text-gray-600 leading-relaxed",
                        "All processing occurs locally on your computer"
                    }
                }
            }

            // Why RoboAMO Section
            div {
                class: "mt-8 py-16 md:py-20 px-4 sm:px-6 lg:px-8 bg-gradient-to-b from-white to-gray-50 max-w-7xl mx-auto",
                div {
                    class: "mb-12",
                    h2 { class: "text-3xl md:text-4xl font-bold text-gray-900 mb-4", "Why RoboAMO?" }
                    p { class: "text-lg md:text-xl text-gray-600 max-w-2xl",
                        "Data-driven personnel decisions that enhance mission readiness"
                    }
                }

                div {
                    class: "grid grid-cols-1 lg:grid-cols-2 gap-6 max-w-6xl mx-auto",
                    div {
                        class: "bg-white rounded-xl p-6 shadow-md",
                        h4 { class: "font-bold text-gray-900 mb-3 text-lg", "Operational Questions" }
                        p { class: "text-gray-600 mb-3", "Have you ever asked yourself:" }
                        ul {
                            class: "list-disc list-inside space-y-1 text-sm text-gray-600 leading-relaxed ml-4",
                            li { "Can we support a day and night check?" }
                            li { "Can we handle a dedicated line shack?" }
                            li { "What if we had to man an additional det tomorrow?" }
                            li { "Do we have the personnel for a dedicated phase shop?" }
                            li { "Should we prioritize CDI training based on gaps?" }
                        }
                    }
                    div {
                        class: "bg-white rounded-xl p-6 shadow-md",
                        h4 { class: "font-bold text-gray-900 mb-3 text-lg", "Strategic Benefits" }
                        p { class: "text-gray-600 mb-3", "RoboAMO helps leaders:" }
                        ul {
                            class: "list-disc list-inside space-y-1 text-sm text-gray-600 leading-relaxed ml-4",
                            li { "Reduce assignment planning time from days to minutes" }
                            li { "Make data-driven decisions for strategic workforce planning" }
                            li { "Prevent mission-critical manning gaps before they occur" }
                            li { "Optimize personnel utilization and operational readiness" }
                            li { "Identify training gaps months ahead of critical shortfalls" }
                        }
                    }
                }
            }

            // How It Works Section
            div {
                id: "how-it-works",
                class: "how-it-works py-16 md:py-20 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto scroll-mt-16",
                h2 { class: "section-title text-3xl md:text-4xl font-bold text-gray-900 mb-4", "How It Works" }
                p { class: "section-subtitle text-lg md:text-xl text-gray-600 mb-12 max-w-2xl",
                    "Four simple uploads provide comprehensive manning insights"
                }

                div {
                    class: "steps-container grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6",
                    div {
                        class: "step-card bg-white rounded-xl p-6 shadow-lg hover:shadow-xl transition-all duration-300 border-t-4 border-blue-500 relative",
                        div { class: "step-number absolute -top-4 -left-4 w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-600 text-white rounded-full flex items-center justify-center font-bold text-lg shadow-md", "1" }
                        div {
                            class: "mt-2",
                            h4 { class: "step-title font-bold text-gray-900 mb-2", "Requirements File" }
                            p { class: "step-desc text-sm text-gray-600 mb-3 leading-relaxed",
                                "Team structures and qualification requirements"
                            }
                            div { class: "file-type-badge inline-block px-3 py-1 bg-gray-100 text-gray-700 rounded-md text-xs font-mono font-semibold", ".csv" }
                        }
                    }
                    div {
                        class: "step-card bg-white rounded-xl p-6 shadow-lg hover:shadow-xl transition-all duration-300 border-t-4 border-blue-500 relative",
                        div { class: "step-number absolute -top-4 -left-4 w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-600 text-white rounded-full flex items-center justify-center font-bold text-lg shadow-md", "2" }
                        div {
                            class: "mt-2",
                            h4 { class: "step-title font-bold text-gray-900 mb-2", "Qual Definitions" }
                            p { class: "step-desc text-sm text-gray-600 mb-3 leading-relaxed",
                                "Maps common names to ASM equivalents"
                            }
                            div { class: "file-type-badge inline-block px-3 py-1 bg-gray-100 text-gray-700 rounded-md text-xs font-mono font-semibold", ".csv" }
                        }
                    }
                    div {
                        class: "step-card bg-white rounded-xl p-6 shadow-lg hover:shadow-xl transition-all duration-300 border-t-4 border-blue-500 relative",
                        div { class: "step-number absolute -top-4 -left-4 w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-600 text-white rounded-full flex items-center justify-center font-bold text-lg shadow-md", "3" }
                        div {
                            class: "mt-2",
                            h4 { class: "step-title font-bold text-gray-900 mb-2", "ASM Report" }
                            p { class: "step-desc text-sm text-gray-600 mb-3 leading-relaxed",
                                "Personnel and their qualifications"
                            }
                            div { class: "file-type-badge inline-block px-3 py-1 bg-gray-100 text-gray-700 rounded-md text-xs font-mono font-semibold", ".xlsx" }
                        }
                    }
                    div {
                        class: "step-card bg-white rounded-xl p-6 shadow-lg hover:shadow-xl transition-all duration-300 border-t-4 border-blue-500 relative",
                        div { class: "step-number absolute -top-4 -left-4 w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-600 text-white rounded-full flex items-center justify-center font-bold text-lg shadow-md", "4" }
                        div {
                            class: "mt-2",
                            h4 { class: "step-title font-bold text-gray-900 mb-2", "FLTMPS Roster" }
                            p { class: "step-desc text-sm text-gray-600 mb-3 leading-relaxed",
                                "PRD and duty status information"
                            }
                            div { class: "file-type-badge inline-block px-3 py-1 bg-gray-100 text-gray-700 rounded-md text-xs font-mono font-semibold", ".xlsx" }
                        }
                    }
                }
            }

            // Methodology Section
            div {
                class: "methodology-section py-16 md:py-20 px-4 sm:px-6 lg:px-8 bg-gradient-to-b from-white to-gray-50 max-w-7xl mx-auto",
                div {
                    class: "methodology-header mb-12",
                    h2 { class: "section-title text-3xl md:text-4xl font-bold text-gray-900 mb-4", "Intelligent Optimization" }
                    p { class: "section-subtitle text-lg md:text-xl text-gray-600 max-w-2xl",
                        "Advanced algorithm maximizes readiness while respecting operational constraints"
                    }
                }

                div {
                    class: "methodology-grid grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 max-w-6xl mx-auto",
                    div {
                        class: "methodology-card bg-white rounded-xl p-6 text-center shadow-md hover:shadow-xl hover:scale-105 transition-all duration-300",
                        div { class: "methodology-icon text-3xl md:text-4xl mb-3", "🎖️" }
                        h4 { class: "methodology-title font-bold text-gray-900 mb-2 text-base md:text-lg", "TAR Priority" }
                        p { class: "methodology-desc text-sm text-gray-600 leading-relaxed",
                            "Active duty first, then SELRES"
                        }
                    }
                    div {
                        class: "methodology-card bg-white rounded-xl p-6 text-center shadow-md hover:shadow-xl hover:scale-105 transition-all duration-300",
                        div { class: "methodology-icon text-3xl md:text-4xl mb-3", "✈️" }
                        h4 { class: "methodology-title font-bold text-gray-900 mb-2 text-base md:text-lg", "Aircrew Protection" }
                        p { class: "methodology-desc text-sm text-gray-600 leading-relaxed",
                            "Preserves flight crew availability"
                        }
                    }
                    div {
                        class: "methodology-card bg-white rounded-xl p-6 text-center shadow-md hover:shadow-xl hover:scale-105 transition-all duration-300",
                        div { class: "methodology-icon text-3xl md:text-4xl mb-3", "📅" }
                        h4 { class: "methodology-title font-bold text-gray-900 mb-2 text-base md:text-lg", "Rotation Planning" }
                        p { class: "methodology-desc text-sm text-gray-600 leading-relaxed",
                            "Prioritizes 12+ months remaining"
                        }
                    }
                    div {
                        class: "methodology-card bg-white rounded-xl p-6 text-center shadow-md hover:shadow-xl hover:scale-105 transition-all duration-300",
                        div { class: "methodology-icon text-3xl md:text-4xl mb-3", "⚓" }
                        h4 { class: "methodology-title font-bold text-gray-900 mb-2 text-base md:text-lg", "Leadership Reserve" }
                        p { class: "methodology-desc text-sm text-gray-600 leading-relaxed",
                            "Junior sailors first"
                        }
                    }
                }
            }

            // Disclaimer
            div {
                class: "disclaimer-section py-8 md:py-12 px-4 sm:px-6 lg:px-8 bg-blue-50 border-t border-blue-100",
                div {
                    class: "disclaimer-content max-w-4xl mx-auto text-center",
                    h3 { class: "disclaimer-title text-lg md:text-xl font-bold text-blue-900 mb-3", "Strategic Planning Tool" }
                    p { class: "disclaimer-text text-sm md:text-base text-blue-800 leading-relaxed",
                        "RoboAMO provides data-driven recommendations. Final decisions should incorporate leadership judgment and operational requirements."
                    }
                }
            }
        }
    }
}
