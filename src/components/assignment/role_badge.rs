use dioxus::prelude::*;

#[component]
pub fn RoleBadge(qualification: String, is_locked: bool) -> Element {
    rsx! {
        button {
            class: if is_locked {
                "role-badge role-badge--locked cursor-pointer hover:bg-yellow-200 transition-all duration-150 active:transform active:scale-95"
            } else {
                "role-badge cursor-pointer hover:bg-gray-200 transition-all duration-150 active:transform active:scale-95"
            },
            onclick: move |_| {
                // TODO: Open assignment modal
            },
            "{qualification}"
        }
    }
}
