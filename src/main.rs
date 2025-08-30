// The dioxus prelude contains a ton of common items used in dioxus apps. It's a good idea to import wherever you
// need dioxus
use dioxus::prelude::*;

use utilities::config::AppState;
use views::{FileUpload, Home, Navbar, ProductRoadmap, Results};

mod components;
mod engine;
mod utilities;
mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {

    #[layout(Navbar)]

        #[route("/")]
        Home { },

        #[route("/file_upload/:page")]

        FileUpload { page: String },

        #[route("/results")]

        Results { },

        #[route("/roadmap")]

        ProductRoadmap { },
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(AppState::default()));
    rsx! {
        document::Stylesheet {
            href: TAILWIND_CSS,
        }
        document::Link { rel: "icon", href: "data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>🔧</text></svg>" }

        div {
            Router::<Route> {}
        }

    }
}
