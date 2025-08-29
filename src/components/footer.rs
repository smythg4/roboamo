use dioxus::prelude::*;

static ICON: Asset = asset!("/assets/favicon.ico");

#[component]
pub fn Footer() -> Element {
    rsx! {
        div {
            class: "flex justify-center align-middle p-2",
            a {
                class: "flex",
                href: "https://dioxuslabs.com/",
                target: "_blank",
                rel: "noopener noreferrer",
                img {
                    class: "max-h-8",
                    src: ICON,
                }
                p {
                    class: "max-h-8",
                    "Built with Dioxus"
                }
            }
        }

    }
}
