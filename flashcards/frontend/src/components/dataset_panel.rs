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
    pub show_import: bool,
    pub show_export: bool,
    pub on_file_select: Callback<Event>,
    pub on_download: Callback<MouseEvent>,
}

#[function_component(DatasetPanel)]
pub fn dataset_panel(props: &DatasetPanelProps) -> Html {
    let dataset_list = if props.datasets.is_empty() {
        html! { <p class="muted-note">{"No wordsets yet. Create one below."}</p> }
    } else {
        html! {
            <div class="dataset-list">
                { for props.datasets.iter().map(|dataset| {
                    let name = dataset.name.clone();
                    let name_for_delete = dataset.name.clone();
                    let is_selected = props.current_dataset == dataset.name;
                    let on_select_dataset = props.on_select_dataset.clone();
                    let on_delete_dataset = props.on_delete_dataset.clone();
                    let select_class = if is_selected {
                        "btn dataset-btn is-selected"
                    } else {
                        "btn dataset-btn"
                    };

                    html! {
                        <div key={dataset.name.clone()} class="dataset-item">
                            <button
                                onclick={Callback::from(move |_| on_select_dataset.emit(name.clone()))}
                                class={select_class}
                            >
                                { &dataset.name }
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
            { if props.show_export {
                html! {
                    <div class="panel-actions">
                        <button class="btn btn-secondary" onclick={props.on_download.clone()}>
                            {"Export Flashcards"}
                        </button>
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
