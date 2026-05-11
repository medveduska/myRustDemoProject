use web_sys::{Event, InputEvent, MouseEvent};
use yew::prelude::*;

use crate::model::Dataset;

#[derive(Properties, PartialEq)]
pub struct DatasetPanelProps {
    pub datasets: Vec<Dataset>,
    pub current_dataset: String,
    pub show_dataset_input: bool,
    pub new_dataset_name: String,
    pub on_select_dataset: Callback<String>,
    pub on_delete_dataset: Callback<String>,
    pub on_toggle_input: Callback<MouseEvent>,
    pub on_dataset_name_input: Callback<InputEvent>,
    pub on_create_dataset: Callback<MouseEvent>,
    pub on_save_all: Callback<MouseEvent>,
    pub on_open_backup_picker: Callback<MouseEvent>,
    pub on_backup_file_select: Callback<Event>,
    pub backup_status_message: Option<String>,
    pub backup_status_is_error: bool,
    pub show_import: bool,
    pub show_export: bool,
    pub on_file_select: Callback<Event>,
    pub on_download: Callback<MouseEvent>,
    pub renaming_dataset: Option<String>,
    pub rename_input: String,
    pub on_start_rename: Callback<String>,
    pub on_rename_input: Callback<InputEvent>,
    pub on_confirm_rename: Callback<MouseEvent>,
    pub on_cancel_rename: Callback<MouseEvent>,
}

#[function_component(DatasetPanel)]
pub fn dataset_panel(props: &DatasetPanelProps) -> Html {
    let backup_status = props.backup_status_message.as_ref().map(|message| {
        let class_name = if props.backup_status_is_error {
            "backup-status is-error"
        } else {
            "backup-status"
        };

        html! {
            <p class={class_name}>{message}</p>
        }
    });

    let dataset_list = if props.datasets.is_empty() {
        html! { <p class="muted-note">{"No wordsets yet. Create one below."}</p> }
    } else {
        html! {
            <div class="dataset-list">
                { for props.datasets.iter().map(|dataset| {
                    let name = dataset.name.clone();
                    let is_selected = props.current_dataset == dataset.name;
                    let is_renaming = props.renaming_dataset.as_deref() == Some(dataset.name.as_str());
                    let on_select_dataset = props.on_select_dataset.clone();
                    let on_delete_dataset = props.on_delete_dataset.clone();
                    let on_start_rename = props.on_start_rename.clone();
                    let on_confirm_rename = props.on_confirm_rename.clone();
                    let on_cancel_rename = props.on_cancel_rename.clone();
                    let on_rename_input = props.on_rename_input.clone();
                    let rename_input = props.rename_input.clone();
                    let select_class = if is_selected {
                        "btn dataset-btn is-selected"
                    } else {
                        "btn dataset-btn"
                    };

                    if is_renaming {
                        let name_for_delete = name.clone();
                        html! {
                            <div key={name} class="dataset-item">
                                <div class="inline-rename-row">
                                    <input
                                        type="text"
                                        value={rename_input}
                                        oninput={on_rename_input}
                                        class="text-input"
                                    />
                                    <button class="btn btn-primary" onclick={on_confirm_rename}>{"Save"}</button>
                                    <button class="btn btn-secondary" onclick={on_cancel_rename}>{"Cancel"}</button>
                                </div>
                                <button
                                    onclick={Callback::from(move |_| on_delete_dataset.emit(name_for_delete.clone()))}
                                    class="dataset-delete-subaction"
                                    title="Delete this wordset"
                                >
                                    { "Delete" }
                                </button>
                            </div>
                        }
                    } else {
                        let name_for_select = name.clone();
                        let name_for_rename = name.clone();
                        let name_for_delete = name.clone();
                        html! {
                            <div key={name} class="dataset-item">
                                <button
                                    onclick={Callback::from(move |_| on_select_dataset.emit(name_for_select.clone()))}
                                    class={select_class}
                                >
                                    { &dataset.name }
                                </button>
                                <button
                                    onclick={Callback::from(move |_| on_start_rename.emit(name_for_rename.clone()))}
                                    class="dataset-rename-subaction"
                                    title="Rename this wordset"
                                >
                                    { "Rename" }
                                </button>
                                <button
                                    onclick={Callback::from(move |_| on_delete_dataset.emit(name_for_delete.clone()))}
                                    class="dataset-delete-subaction"
                                    title="Delete this wordset"
                                >
                                    { "Delete" }
                                </button>
                            </div>
                        }
                    }
                }) }
            </div>
        }
    };

    html! {
        <section class="panel">
            <h3 class="panel-title">{"Wordsets"}</h3>
            <div class="panel-content">
                { dataset_list }
            </div>
            <div class="panel-actions">
                <button class="btn btn-secondary" onclick={props.on_toggle_input.clone()}>
                    { if props.show_dataset_input { "Cancel" } else { "New Wordset" } }
                </button>
            </div>
            { if props.show_dataset_input {
                html! {
                    <div class="inline-create-row">
                        <input
                            type="text"
                            placeholder="Wordset name (e.g., 'HSK 1', 'Business Terms')"
                            value={props.new_dataset_name.clone()}
                            oninput={props.on_dataset_name_input.clone()}
                            class="text-input"
                        />
                        <button class="btn btn-primary" onclick={props.on_create_dataset.clone()}>{"Create"}</button>
                    </div>
                }
            } else {
                html! {}
            } }
            <div class="action-section">
                <p class="action-title">{"Full Progress Backup"}</p>
                <div class="action-button-row">
                    <button class="btn btn-secondary" onclick={props.on_save_all.clone()}>
                        {"Save All"}
                    </button>
                    <button class="btn btn-secondary" onclick={props.on_open_backup_picker.clone()}>
                        {"Load"}
                    </button>
                </div>
                <input
                    id="load-progress-input"
                    class="visually-hidden-input"
                    type="file"
                    accept=".json,application/json"
                    onchange={props.on_backup_file_select.clone()}
                />
                <p class="action-description">
                    {"Save All downloads every wordset and your current study position. Load replaces the current browser data with a backup file."}
                </p>
                { backup_status.unwrap_or_default() }
            </div>
            { if props.show_export {
                html! {
                    <div class="action-section">
                        <p class="action-title">{"Export this wordset"}</p>
                        <div class="action-button-row">
                            <button class="btn btn-secondary" onclick={props.on_download.clone()}>
                                {"Export Flashcards"}
                            </button>
                        </div>
                        <p class="action-description">
                            {"Downloads the active wordset (all cards and their known/unknown status) as a CSV file."}
                        </p>
                    </div>
                }
            } else {
                html! {}
            } }
            { if props.show_import {
                html! {
                    <div class="import-group">
                        <label class="input-label" for="import-flashcards-input">{"Import Flashcards"}</label>
                        <input
                            id="import-flashcards-input"
                            class="file-input"
                            type="file"
                            accept=".csv"
                            onchange={props.on_file_select.clone()}
                        />
                    </div>
                }
            } else {
                html! {}
            } }
        </section>
    }
}
