use dioxus::prelude::*;
use crate::utilities::{parse_asm_file, parse_fltmps_file, parse_qual_defs, parse_requirements, PreviewType, handle_result};
use crate::Route;
use crate::components::Preview;
use crate::utilities::config::{AppState, ParsedData};
use crate::views::ErrorDisplay;

use std::rc::Rc;
use anyhow::{Result, Context};

#[component]
pub fn FileUpload(page: String) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let nav = navigator();
    
    let mut error_state = use_signal(|| None::<String>);
    let page_data_result = (|| -> Result<_> {
        let app_state_read = app_state();
        let page_data = app_state_read.files.get(&page)
            .context(format!("Configuration for page '{}' not found", page))?;
        Ok((
            page_data.page_desc.clone(),
            page_data.file_types.clone(),
            page_data.next_page.clone(),
            page_data.file_content.clone(),
            page_data.file_name.clone(),
            page_data.preview_type.clone(),
        ))
    })();

    let (page_desc, file_types, next_page, file_content, file_name, preview_type) = match page_data_result {
        Ok(data) => data,
        Err(e) => {
            return rsx! {
                ErrorDisplay {
                    error: format!("Failed to load page configuration: {:#}", e)
                }
            };
        }
    };

    let has_file = file_content.is_some();

    // Determine step number for display
    let step_num = match page.as_str() {
        "Requirements" => 1,
        "Qual Defs" => 2,
        "ASM" => 3,
        "FLTMPS" => 4,
        _ => 0,
    };

    // Clone page for use in closures
    let page_for_onchange = page.clone();
    let page_for_onclick = page.clone();

    rsx! {
        div {
            class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8",
            
            if let Some(error) = error_state() {
                div {
                    class: "mb-4",
                    ErrorDisplay {
                        error: error.clone(),
                        retry: None,
                    }
                }
            }

            div {
                class: "grid grid-cols-1 lg:grid-cols-3 gap-8",
                
                // Left side - Upload section
                div {
                    class: "lg:col-span-1",
                    div {
                        class: "bg-white rounded-xl shadow-lg p-6",
                        
                        // Step indicator
                        div {
                            class: "flex items-center mb-6",
                            div {
                                class: "flex items-center justify-center w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-600 text-white rounded-full font-bold text-lg shadow-md",
                                "{step_num}"
                            }
                            div {
                                class: "ml-3",
                                h2 {
                                    class: "text-xl font-bold text-gray-900",
                                    "Upload {page} File"
                                }
                            }
                        }
                        
                        // Description
                        p {
                            class: "text-gray-600 mb-6 leading-relaxed",
                            "{page_desc}"
                        }
                        
                        // File type badge
                        div {
                            class: "mb-4",
                            span {
                                class: "inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-700",
                                "Expected format: "
                                span {
                                    class: "ml-1 font-mono font-bold",
                                    "{file_types}"
                                }
                            }
                        }
                        
                        // Upload area
                        div {
                            class: "relative",
                            input {
                                r#type: "file",
                                id: "file-{page}",
                                class: "hidden",
                                key: "{page}",
                                accept: "{file_types}",
                                multiple: false,
                                onchange: move |evt| {
                                    let page = page_for_onchange.clone();
                                    async move {
                                        let result: Result<()> = async {
                                            let file_engine = evt.files()
                                                .context("No file engine available")?;

                                            let files = file_engine.files();
                                            for fname in &files {
                                                let file = file_engine.read_file(fname).await
                                                    .context(format!("Failed to read file: {}", fname))?;

                                                app_state.write().files.get_mut(&page)
                                                    .context("Page not found in state")?
                                                    .file_content = Some(Rc::new(file));
                                            }
                                            Ok(())
                                        }.await;

                                        if let Err(e) = result {
                                            error_state.set(Some(format!("Upload failed: {:#}", e)));
                                        }
                                    }
                                }
                            }
                            
                            label {
                                r#for: "file-{page}",
                                class: if has_file {
                                    "flex flex-col items-center justify-center w-full h-32 border-2 border-dashed rounded-lg cursor-pointer transition-all duration-200 border-green-300 bg-green-50 hover:bg-green-100"
                                } else {
                                    "flex flex-col items-center justify-center w-full h-32 border-2 border-dashed rounded-lg cursor-pointer transition-all duration-200 border-gray-300 bg-gray-50 hover:bg-gray-100 hover:border-blue-400"
                                },
                                
                                if has_file {
                                    // File uploaded state
                                    div {
                                        class: "flex flex-col items-center",
                                        svg {
                                            class: "w-10 h-10 mb-2 text-green-500",
                                            fill: "none",
                                            stroke: "currentColor",
                                            "stroke-width": "2",
                                            "viewBox": "0 0 24 24",
                                            path {
                                                "stroke-linecap": "round",
                                                "stroke-linejoin": "round",
                                                d: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                                            }
                                        }
                                        p {
                                            class: "text-sm font-medium text-gray-900",
                                            "{file_name.clone().unwrap_or_default()}"
                                        }
                                        p {
                                            class: "text-xs text-gray-500 mt-1",
                                            "Click to replace"
                                        }
                                    }
                                } else {
                                    // No file state
                                    div {
                                        class: "flex flex-col items-center",
                                        svg {
                                            class: "w-10 h-10 mb-2 text-gray-400",
                                            fill: "none",
                                            stroke: "currentColor",
                                            "stroke-width": "2",
                                            "viewBox": "0 0 24 24",
                                            path {
                                                "stroke-linecap": "round",
                                                "stroke-linejoin": "round",
                                                d: "M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
                                            }
                                        }
                                        p {
                                            class: "text-sm font-medium text-gray-900",
                                            "Click to upload"
                                        }
                                        p {
                                            class: "text-xs text-gray-500 mt-1",
                                            "or drag and drop"
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Status message
                        if has_file {
                            div {
                                class: "mt-4 p-3 bg-green-100 border border-green-200 rounded-lg",
                                p {
                                    class: "text-sm text-green-800 font-medium flex items-center",
                                    svg {
                                        class: "w-4 h-4 mr-2 flex-shrink-0",
                                        fill: "currentColor",
                                        "viewBox": "0 0 20 20",
                                        path {
                                            "fill-rule": "evenodd",
                                            d: "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z",
                                            "clip-rule": "evenodd"
                                        }
                                    }
                                    "File uploaded successfully"
                                }
                            }
                        }
                    }
                }
                
                // Right side - Preview section
                if has_file {
                    div {
                        class: "lg:col-span-2",
                        div {
                            class: "bg-white rounded-xl shadow-lg p-6",
                            
                            // Preview header
                            div {
                                class: "flex justify-between items-center mb-6",
                                div {
                                    h2 {
                                        class: "text-2xl font-bold text-gray-900",
                                        "Preview"
                                    }
                                    p {
                                        class: "text-sm text-gray-500 mt-1",
                                        "Review your data before proceeding"
                                    }
                                }
                                
                                // Action buttons
                                div {
                                    class: "flex gap-3",
                                    
                                    // Re-upload button
                                    label {
                                        r#for: "file-{page}",
                                        class: "flex items-center gap-2 px-4 py-2 text-sm font-medium text-gray-600 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition-all duration-200 cursor-pointer",
                                        svg {
                                            class: "w-4 h-4",
                                            fill: "none",
                                            stroke: "currentColor",
                                            "stroke-width": "2",
                                            "viewBox": "0 0 24 24",
                                            path {
                                                "stroke-linecap": "round",
                                                "stroke-linejoin": "round",
                                                d: "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
                                            }
                                        }
                                        "Replace File"
                                    }
                                    
                                    // Confirm button
                                    button {
                                        onclick: move |_| {
                                            let page = page_for_onclick.clone();
                                            
                                            if let Some(entry) = app_state.write().files.get_mut(&page) {
                                                if let Some(file) = &entry.file_content {
                                                    let file = file.clone();
                                                    let parsed_data = match entry.preview_type {
                                                        PreviewType::Requirements => parse_requirements(file).ok()
                                                            .map(Rc::new)
                                                            .map(ParsedData::Requirements),
                                                        PreviewType::QualDef => parse_qual_defs(file).ok()
                                                            .map(Rc::new)
                                                            .map(ParsedData::QualDefs),
                                                        PreviewType::ASM => parse_asm_file(file).ok()
                                                            .map(Rc::new)
                                                            .map(ParsedData::ASM),
                                                        PreviewType::FLTMPS => parse_fltmps_file(file).ok()
                                                            .map(Rc::new)
                                                            .map(ParsedData::FLTMPS),
                                                    };
                                                    entry.parsed_data = parsed_data;
                                                }
                                            }
                                            
                                            if let Some(next) = next_page.clone() {
                                                nav.push(Route::FileUpload { page: next });
                                            }
                                        },
                                        class: "flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-green-700 to-green-400 text-white font-medium rounded-lg hover:from-green-600 hover:to-green-300 shadow-md hover:shadow-lg transform hover:-translate-y-0.5 transition-all duration-200",
                                        svg {
                                            class: "w-4 h-4",
                                            fill: "none",
                                            stroke: "currentColor",
                                            "stroke-width": "2",
                                            "viewBox": "0 0 24 24",
                                            path {
                                                "stroke-linecap": "round",
                                                "stroke-linejoin": "round",
                                                d: "M5 13l4 4L19 7"
                                            }
                                        }
                                        "Looks Good"
                                        if next_page.is_some() {
                                            svg {
                                                class: "w-4 h-4 ml-1",
                                                fill: "none",
                                                stroke: "currentColor",
                                                "stroke-width": "2",
                                                "viewBox": "0 0 24 24",
                                                path {
                                                    "stroke-linecap": "round",
                                                    "stroke-linejoin": "round",
                                                    d: "M9 5l7 7-7 7"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Preview content with scroll
                            div {
                                class: "border border-gray-200 rounded-lg overflow-hidden",
                                div {
                                    class: "max-h-[600px] overflow-y-auto bg-gray-50 p-4",
                                    Preview { preview_type: preview_type.clone() }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}