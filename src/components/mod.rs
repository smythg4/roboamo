//! The components module contains all shared components for our app. Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals. In this template, we define a Hero
//! component  to be used in our app.

mod hero;
pub use hero::Hero;

mod progressbar;
pub use progressbar::ProgressBar;

mod preview;
pub use preview::Preview;

mod search_bar;
pub use search_bar::SearchBar;

mod interaction_bar;
pub use interaction_bar::{InteractionAction, InteractionBar, InteractionMode};

mod analysis_date_bar;
pub use analysis_date_bar::AnalysisDateBar;

mod footer;
pub use footer::Footer;

mod player_card;
pub use player_card::PlayerCard;

// Domain-specific component modules
pub mod assignment;
pub use assignment::{AssignmentStats, RoleBadge, TeamCard, TeamRow, UnassignedTable};
