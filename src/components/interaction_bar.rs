use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum InteractionMode {
    ViewOnly,
    Swap,
    Lock,
}

#[derive(Clone, PartialEq)]
pub enum InteractionAction {
    SetMode(InteractionMode),
    ExecuteSwap,
    ExecuteLock,
    ClearLocks,
    SaveState,
}

#[component]
pub fn InteractionBar(
    interaction_mode_signal: Signal<InteractionMode>,
    selected_count_signal: ReadOnlySignal<usize>,
    persistent_locks_count_signal: ReadOnlySignal<usize>,
    on_action: EventHandler<InteractionAction>,
) -> Element {
    // Read current values from signals - component will auto-rerender when these change
    let interaction_mode = interaction_mode_signal();
    let selected_count = selected_count_signal();
    let persistent_locks_count = persistent_locks_count_signal();
    rsx! {
        div {
            class: "sticky top-17 z-50 bg-white shadow-md border-b border-gray-200 flex gap-2 p-4 w-175",
            button {
                class: match interaction_mode {
                    InteractionMode::ViewOnly => "px-4 py-2 bg-gray-600 text-white rounded-lg font-medium",
                    _ => "px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
                },
                onclick: move |_| {
                    on_action.call(InteractionAction::SetMode(InteractionMode::ViewOnly));
                },
                "👁️ View Only"
            }
            // Swap Mode - button changes when selections are ready
            button {
                class: match (interaction_mode, selected_count) {
                    (InteractionMode::Swap, 2) => "px-4 py-2 bg-blue-600 text-white rounded-lg font-medium animate-pulse",
                    (InteractionMode::Swap, _) => "px-4 py-2 bg-blue-500 text-white rounded-lg font-medium",
                    _ => "px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300",
                },
                onclick: move |_| {
                    if interaction_mode == InteractionMode::Swap && selected_count == 2 {
                        on_action.call(InteractionAction::ExecuteSwap);
                    } else {
                        on_action.call(InteractionAction::SetMode(InteractionMode::Swap));
                    }
                },
                match (interaction_mode, selected_count) {
                    (InteractionMode::Swap, 2) => "🔄 Execute Swap",
                    (InteractionMode::Swap, 1) => "🔄 Select One More",
                    (InteractionMode::Swap, _) => "🔄 Swap Mode",
                    _ => "🔄 Swap Mode"
                }
            }

            // Lock Mode - button changes when selections exist
            button {
                class: match (interaction_mode, selected_count) {
                    (InteractionMode::Lock, n) if n > 0 => "px-4 py-2 bg-orange-600 text-white rounded-lg font-medium animate-pulse",
                    (InteractionMode::Lock, _) => "px-4 py-2 bg-orange-500 text-white rounded-lg font-medium",
                    _ => "px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
                },
                onclick: move |_| {
                    if interaction_mode == InteractionMode::Lock && selected_count > 0 {
                        on_action.call(InteractionAction::ExecuteLock);
                    } else {
                        on_action.call(InteractionAction::SetMode(InteractionMode::Lock));
                    }
                },
                match (interaction_mode, selected_count) {
                    (InteractionMode::Lock, n) if n > 0 => format!("🔒 Lock {} Assignments", n),
                    (InteractionMode::Lock, _) => "🔒 Lock Mode".to_string(),
                    _ => "🔒 Lock Mode".to_string()
                }
            }

            // button to clear all locked selections
            button {
                class: "px-3 py-1 bg-red-500 text-white rounded text-sm hover:bg-red-600",
                onclick: move |_| {
                    on_action.call(InteractionAction::ClearLocks);
                },
                "Clear All Locks ({persistent_locks_count})"
            }

            // button to save current state
            button {
                class: "px-3 py-1 bg-blue-600 text-white rounded text-sm hover:bg-blue-700",
                onclick: move |_| {
                    on_action.call(InteractionAction::SaveState);
                },
                "💾 Save State"
            }
        }
    }
}
