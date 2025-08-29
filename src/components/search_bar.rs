
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SearchBarProps {
    pub placeholder: String,
    pub value: String,
    pub onchange: EventHandler<String>,
    #[props(default = "border border-gray-300 rounded px-3 py-2 mb-4 w-full".to_string())]
    pub class: String,
}

#[component]
pub fn SearchBar(props: SearchBarProps) -> Element {
    rsx! {
        div {
            class: "flex items-center gap-2 mb-4",
            span {
                class: "text-gray-500",
                "🔍"
            }
            input {
                r#type: "text",
                placeholder: "{props.placeholder}",
                class: "{props.class}",
                value: "{props.value}",
                oninput: move |evt| {
                    props.onchange.call(evt.value());
                }
            }
            if !props.value.is_empty() {
                button {
                    class: "text-gray-400 hover:text-gray-600",
                    onclick: move |_| props.onchange.call(String::new()),
                    "✕"
                }
            }
        }
    }
}
