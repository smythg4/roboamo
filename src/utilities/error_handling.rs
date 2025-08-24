// src/utils/component_result.rs
use dioxus::prelude::*;
use anyhow::Result;
use crate::views::ErrorDisplay;


/// Helper to handle Results in components
pub fn handle_result<T, F>(result: Result<T>, success: F) -> Element 
where
    F: FnOnce(T) -> Element,
{
    match result {
        Ok(value) => success(value),
        Err(err) => rsx! {
            ErrorDisplay { 
                error: format!("{:#}", err)
            }
        }
    }
}