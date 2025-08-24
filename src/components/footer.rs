use dioxus::prelude::*;

static ICON: Asset = asset!("/assets/favicon.ico");

#[component]
pub fn Footer() -> Element {
    rsx! {
        div {
            class: "flex justify-center align-middle p-2",
            Link {
                class: "flex",
                to: "https://dioxuslabs.com/",
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
