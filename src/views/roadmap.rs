use crate::components::Footer;
use dioxus::prelude::*;

#[component]
pub fn ProductRoadmap() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 py-12 px-4 sm:px-6 lg:px-8",

            div {
                class: "max-w-7xl mx-auto",

                // Header Section
                div {
                    class: "text-center mb-12",
                    h1 {
                        class: "text-4xl font-bold text-gray-900 mb-4",
                        "Product Roadmap"
                    }
                    p {
                        class: "text-xl text-gray-600 max-w-3xl mx-auto",
                        "Continuous improvements and feature development for RoboAMO"
                    }
                }

                // Two Column Layout
                div {
                    class: "grid grid-cols-1 lg:grid-cols-2 gap-8",

                    // Future Features Column
                    div {
                        class: "bg-white rounded-xl shadow-lg overflow-hidden",
                        div {
                            class: "bg-gradient-to-r from-blue-500 to-indigo-600 p-6",
                            h2 {
                                class: "text-2xl font-bold text-white flex items-center",
                                span { class: "mr-3 text-3xl", "🚀" }
                                "Future Features"
                            }
                        }
                        div {
                            class: "p-6 space-y-4",

                            // MAJOR COMPLETED FEATURES
                            FeatureCard {
                                icon: "🎯",
                                title: "Role-Based Position Tracking",
                                description: "Eliminated single assignment lock limitation with granular position instances and intelligent swap eligibility highlighting",
                                status: "complete"
                            }
                            FeatureCard {
                                icon: "🔄",
                                title: "Advanced Assignment Management",
                                description: "Full swap/lock system with intelligent eligibility highlighting - fantasy football-inspired roster management UI to come",
                                status: "complete"
                            }

                            // CURRENT PRIORITIES
                            FeatureCard {
                                icon: "💾",
                                title: "Save State System",
                                description: "Export/import complete analysis sessions as JSON for backup, collaboration, and audit trails",
                                status: "complete"
                            }
                            FeatureCard {
                                icon: "👤",
                                title: "PersonCard Tooltips",
                                description: "Hover over personnel names to see detailed cards with qualifications, status, and current assignment",
                                status: "complete"
                            }
                            FeatureCard {
                                icon: "📈",
                                title: "12-Month Manning Projection",
                                description: "Strategic time-series visualization showing unfilled positions and SELRES requirements as TAR personnel rotate out",
                                status: "planned"
                            }

                            // STRATEGIC FEATURES
                            FeatureCard {
                                icon: "🏢",
                                title: "Multi-Squadron Support",
                                description: "Executive dashboard managing multiple squadrons simultaneously with comparative analytics",
                                status: "planned"
                            }
                            FeatureCard {
                                icon: "🦺",
                                title: "Weighted Team Priorities",
                                description: "Prioritize critical teams over others (e.g. Home Guard takes precedence over Det assignments)",
                                status: "planned"
                            }
                            FeatureCard {
                                icon: "⚙️",
                                title: "Composite Qualification Rules",
                                description: "Define custom qualification combinations (e.g., '200 CDI' = requires both '210 CDI' AND '220 CDI')",
                                status: "research"
                            }
                            FeatureCard {
                                icon: "📝",
                                title: "In-Browser Data Editing",
                                description: "Edit uploaded personnel and requirements data directly in the browser before running analysis",
                                status: "research"
                            }
                        }
                    }

                    // Known Issues Column
                    div {
                        class: "bg-white rounded-xl shadow-lg overflow-hidden",
                        div {
                            class: "bg-gradient-to-r from-amber-500 to-orange-600 p-6",
                            h2 {
                                class: "text-2xl font-bold text-white flex items-center",
                                span { class: "mr-3 text-3xl", "🔧" }
                                "Known Issues"
                            }
                        }
                        div {
                            class: "p-6 space-y-4",

                            // MEDIUM PRIORITY
                            IssueCard {
                                severity: "medium",
                                title: "TAR/SELRES Classification",
                                description: "FLTMPS alpha roster may not be the most reliable source for duty status determination",
                                workaround: "Manual verification recommended for critical assignments"
                            }
                            IssueCard {
                                severity: "medium",
                                title: "Qualification Standards",
                                description: "Need definitive rules for standard qualification definitions (100 CDI, 040 SUP, etc.)",
                                workaround: "Get wing validated standard definitions"
                            }
                            IssueCard {
                                severity: "medium",
                                title: "Expiration Dates",
                                description: "System doesn't currently track qualification expiration dates",
                                workaround: "Ensure all uploaded data reflects current qualifications only"
                            }
                            IssueCard {
                                severity: "medium",
                                title: "File Upload Error Recovery",
                                description: "If any file fails to parse, users must restart the entire 4-step upload process",
                                workaround: "Ensure files match expected format before uploading. Use demo files as templates."
                            }

                            // LOW PRIORITY
                            IssueCard {
                                severity: "low",
                                title: "Qualification Name Matching",
                                description: "ASM qualification names must exactly match qualification table entries (spaces, punctuation, etc.)",
                                workaround: "Verify qual table entries precisely match ASM qualification text. Future fuzzy matching planned."
                            }
                            IssueCard {
                                severity: "low",
                                title: "Browser Memory Limits",
                                description: "Large multi-squadron files may approach browser memory limits",
                                workaround: "Current single-squadron usage well within safe limits. Multi-squadron batching planned."
                            }
                        }
                    }
                }

                Footer {}
            }
        }
    }
}

#[component]
fn FeatureCard(
    icon: &'static str,
    title: &'static str,
    description: &'static str,
    status: &'static str,
) -> Element {
    let status_class = match status {
        "complete" => "bg-green-100 text-green-700",
        "planned" => "bg-gray-100 text-gray-700",
        "in-progress" => "bg-blue-100 text-blue-700",
        "research" => "bg-purple-100 text-purple-700",
        _ => "bg-gray-100 text-gray-700",
    };

    let status_label = match status {
        "complete" => "Complete",
        "planned" => "Planned",
        "in-progress" => "In Progress",
        "research" => "Research",
        _ => "Future",
    };

    rsx! {
        div {
            class: "flex items-start space-x-3 p-4 rounded-lg hover:bg-gray-50 transition-colors",
            div {
                class: "flex-shrink-0 text-2xl",
                {icon}
            }
            div {
                class: "flex-1",
                div {
                    class: "flex items-center justify-between mb-1",
                    h4 {
                        class: "font-semibold text-gray-900",
                        {title}
                    }
                    span {
                        class: format!("px-2 py-1 text-xs font-medium rounded-full {}", status_class),
                        {status_label}
                    }
                }
                p {
                    class: "text-sm text-gray-600",
                    {description}
                }
            }
        }
    }
}

#[component]
fn IssueCard(
    severity: &'static str,
    title: &'static str,
    description: &'static str,
    workaround: &'static str,
) -> Element {
    let severity_class = match severity {
        "high" => "bg-red-100 text-red-700 border-red-200",
        "medium" => "bg-yellow-100 text-yellow-700 border-yellow-200",
        "low" => "bg-blue-100 text-blue-700 border-blue-200",
        _ => "bg-gray-100 text-gray-700 border-gray-200",
    };

    let severity_icon = match severity {
        "high" => "⚠️",
        "medium" => "⚡",
        "low" => "ℹ️",
        _ => "📝",
    };

    rsx! {
        div {
            class: format!("border-l-4 p-4 rounded-r-lg {}", severity_class),
            div {
                class: "flex items-start",
                span {
                    class: "text-xl mr-3",
                    {severity_icon}
                }
                div {
                    h4 {
                        class: "font-semibold text-gray-900 mb-1",
                        {title}
                    }
                    p {
                        class: "text-sm text-gray-700 mb-2",
                        {description}
                    }
                    if !workaround.is_empty() {
                        div {
                            class: "text-xs text-gray-600 italic",
                            span {
                                class: "font-semibold",
                                "Workaround: "
                            }
                            {workaround}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TimelineItem(
    date: &'static str,
    title: &'static str,
    description: &'static str,
    completed: bool,
) -> Element {
    let dot_class = if completed {
        "bg-green-500"
    } else {
        "bg-gray-400"
    };

    rsx! {
        div {
            class: "relative flex items-start mb-8",
            div {
                class: format!("absolute left-6 w-4 h-4 rounded-full {} ring-4 ring-white", dot_class)
            }
            div {
                class: "ml-16",
                div {
                    class: "text-sm font-semibold text-gray-500 mb-1",
                    {date}
                }
                h4 {
                    class: "text-lg font-bold text-gray-900 mb-1",
                    {title}
                }
                p {
                    class: "text-gray-600",
                    {description}
                }
            }
        }
    }
}
