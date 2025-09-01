use dioxus::prelude::*;

#[component]
pub fn RoleBadge(
    qualification: String,
    is_locked: bool,
    on_click: Option<Callback<(f64, f64)>>, // Optional callback with click position
) -> Element {
    rsx! {
        button {
            class: if is_locked {
                "role-badge role-badge--locked cursor-pointer hover:bg-yellow-200 transition-all duration-150 active:transform active:scale-95"
            } else {
                "role-badge cursor-pointer hover:bg-gray-200 transition-all duration-150 active:transform active:scale-95"
            },
            onclick: move |event| {
                if let Some(on_click) = on_click {
                    let coordinates = event.page_coordinates();
                    on_click.call((coordinates.x, coordinates.y));
                }
            },
            "{qualification}"
        }
    }
}
