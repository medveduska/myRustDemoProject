use gloo_file::callbacks::FileReader;
use gloo_file::File;
use rand::seq::SliceRandom;
use rand::thread_rng;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, InputEvent, KeyboardEvent, MouseEvent};
use yew::prelude::*;

use crate::backup_io::{
    backup_file_name, build_backup_snapshot, parse_backup_snapshot, serialize_backup_snapshot,
    trigger_backup_download,
};
use crate::components::add_flashcard_form::AddFlashcardForm;
use crate::components::dataset_panel::DatasetPanel;
use crate::components::flashcard_view::FlashcardView;
use crate::components::help_panel::HelpPanel;
use crate::components::known_cards_table::KnownCardsTable;
use crate::components::study_toolbar::StudyToolbar;
use crate::csv_io::{export_flashcards_csv, parse_flashcards_from_csv, trigger_csv_download};
use crate::model::{Dataset, Flashcard, FlashcardStage, PersistedState, StudyDirection, StudyMode};
use crate::storage::{load_datasets, load_persisted_state, save_datasets, save_persisted_state};

fn split_flashcards(cards: Vec<Flashcard>) -> (Vec<Flashcard>, Vec<Flashcard>) {
    cards.into_iter().partition(|card| card.known)
}

fn display_text(card: &Flashcard, direction: StudyDirection, stage: FlashcardStage) -> String {
    match (direction, stage) {
        (StudyDirection::Normal, FlashcardStage::First) => card.word.clone(),
        (StudyDirection::Normal, FlashcardStage::Second) => card.pinyin.clone().unwrap_or_default(),
        (StudyDirection::Normal, FlashcardStage::Third) => card.translation.clone(),
        (StudyDirection::Reverse, FlashcardStage::First) => card.translation.clone(),
        (StudyDirection::Reverse, FlashcardStage::Second) => {
            card.pinyin.clone().unwrap_or_default()
        }
        (StudyDirection::Reverse, FlashcardStage::Third) => card.word.clone(),
    }
}

fn exercise_prompt(card: &Flashcard, direction: StudyDirection) -> String {
    match direction {
        StudyDirection::Normal => card.word.clone(),
        StudyDirection::Reverse => card.translation.clone(),
    }
}

fn exercise_expected_answer(card: &Flashcard, direction: StudyDirection) -> String {
    match direction {
        StudyDirection::Normal => card.translation.clone(),
        StudyDirection::Reverse => card.word.clone(),
    }
}

fn normalize_english(value: &str) -> String {
    value.trim().to_lowercase()
}

fn normalize_chinese(value: &str) -> String {
    value.trim().to_string()
}

fn is_exercise_answer_correct(input: &str, expected: &str, direction: StudyDirection) -> bool {
    match direction {
        StudyDirection::Normal => normalize_english(input) == normalize_english(expected),
        StudyDirection::Reverse => normalize_chinese(input) == normalize_chinese(expected),
    }
}

fn datasets_with_current_progress(
    datasets: &[Dataset],
    current_dataset: &str,
    flashcards: &[Flashcard],
    known_cards: &[Flashcard],
) -> Vec<Dataset> {
    let mut updated_datasets = datasets.to_vec();

    if current_dataset.is_empty() {
        return updated_datasets;
    }

    if let Some(dataset) = updated_datasets
        .iter_mut()
        .find(|dataset| dataset.name == current_dataset)
    {
        dataset.flashcards = flashcards.to_vec();
        dataset.known_cards = known_cards.to_vec();
    } else {
        updated_datasets.push(Dataset {
            name: current_dataset.to_string(),
            flashcards: flashcards.to_vec(),
            known_cards: known_cards.to_vec(),
        });
    }

    updated_datasets
}

fn normalized_restore_state(mut state: PersistedState) -> PersistedState {
    state.current_index = if state.flashcards.is_empty() {
        0
    } else {
        state
            .current_index
            .min(state.flashcards.len().saturating_sub(1))
    };

    state
}

#[function_component(App)]
pub fn app() -> Html {
    let persisted = load_persisted_state();

    let flashcards = use_state(|| {
        persisted
            .as_ref()
            .map(|state| state.flashcards.clone())
            .unwrap_or_default()
    });
    let known_cards = use_state(|| {
        persisted
            .as_ref()
            .map(|state| state.known_cards.clone())
            .unwrap_or_default()
    });
    let current_index = use_state(|| {
        persisted
            .as_ref()
            .map(|state| state.current_index)
            .unwrap_or(0)
    });
    let stage = use_state(|| {
        persisted
            .as_ref()
            .map(|state| state.stage)
            .unwrap_or_default()
    });
    let direction = use_state(|| {
        persisted
            .as_ref()
            .map(|state| state.direction)
            .unwrap_or_default()
    });
    let reader_handle = use_state(|| None::<FileReader>);

    let datasets = load_datasets();
    let current_dataset = use_state(|| {
        persisted
            .as_ref()
            .map(|state| state.current_dataset.clone())
            .unwrap_or_default()
    });
    let datasets_list = use_state(move || datasets);
    let new_dataset_name = use_state(String::new);
    let show_dataset_input = use_state(|| false);
    let show_add = use_state(|| false);
    let show_help = use_state(|| false);
    let new_word = use_state(String::new);
    let new_pinyin = use_state(String::new);
    let new_translation = use_state(String::new);
    let renaming_dataset = use_state(|| None::<String>);
    let rename_input = use_state(String::new);
    let show_unknown_in_table = use_state(|| false);
    let backup_status_message = use_state(|| None::<String>);
    let backup_status_is_error = use_state(|| false);
    let study_mode = use_state(StudyMode::default);
    let exercise_answer = use_state(String::new);
    let exercise_checked = use_state(|| false);
    let exercise_is_correct = use_state(|| false);

    {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        let direction = direction.clone();
        let current_dataset = current_dataset.clone();

        use_effect_with(
            (
                flashcards.clone(),
                known_cards.clone(),
                current_index.clone(),
                stage.clone(),
                direction.clone(),
                current_dataset.clone(),
            ),
            move |_| {
                save_persisted_state(&PersistedState {
                    flashcards: (*flashcards).clone(),
                    known_cards: (*known_cards).clone(),
                    current_index: *current_index,
                    stage: *stage,
                    direction: *direction,
                    current_dataset: (*current_dataset).clone(),
                });
                || ()
            },
        );
    }

    {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_dataset = current_dataset.clone();
        let datasets_list = datasets_list.clone();

        use_effect_with(
            (
                flashcards.clone(),
                known_cards.clone(),
                current_dataset.clone(),
            ),
            move |_| {
                if !current_dataset.is_empty() {
                    let mut datasets = (*datasets_list).clone();
                    if let Some(dataset) = datasets
                        .iter_mut()
                        .find(|dataset| dataset.name == *current_dataset)
                    {
                        dataset.flashcards = (*flashcards).clone();
                        dataset.known_cards = (*known_cards).clone();
                        datasets_list.set(datasets.clone());
                        save_datasets(&datasets);
                    }
                }
                || ()
            },
        );
    }

    let load_dataset = {
        let datasets_list = datasets_list.clone();
        let current_dataset = current_dataset.clone();
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |name: String| {
            if let Some(dataset) = datasets_list.iter().find(|dataset| dataset.name == name) {
                flashcards.set(dataset.flashcards.clone());
                known_cards.set(dataset.known_cards.clone());
                current_index.set(0);
                stage.set(FlashcardStage::First);
                exercise_answer.set(String::new());
                exercise_checked.set(false);
                current_dataset.set(name);
            }
        })
    };

    let add_new_dataset = {
        let new_dataset_name = new_dataset_name.clone();
        let datasets_list = datasets_list.clone();
        let current_dataset = current_dataset.clone();
        let show_dataset_input = show_dataset_input.clone();
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |_| {
            if !new_dataset_name.is_empty() {
                let mut datasets = (*datasets_list).clone();
                if !datasets
                    .iter()
                    .any(|dataset| dataset.name == *new_dataset_name)
                {
                    datasets.push(Dataset {
                        name: (*new_dataset_name).clone(),
                        flashcards: Vec::new(),
                        known_cards: Vec::new(),
                    });
                    datasets_list.set(datasets.clone());
                    current_dataset.set((*new_dataset_name).clone());
                    flashcards.set(Vec::new());
                    known_cards.set(Vec::new());
                    current_index.set(0);
                    stage.set(FlashcardStage::First);
                    exercise_answer.set(String::new());
                    exercise_checked.set(false);
                    save_datasets(&datasets);
                }
                new_dataset_name.set(String::new());
                show_dataset_input.set(false);
            }
        })
    };

    let oninput_dataset_name = {
        let new_dataset_name = new_dataset_name.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                new_dataset_name.set(input.value());
            }
        })
    };

    let delete_dataset = {
        let datasets_list = datasets_list.clone();
        let current_dataset = current_dataset.clone();
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |name: String| {
            let mut datasets = (*datasets_list).clone();
            datasets.retain(|dataset| dataset.name != name);
            datasets_list.set(datasets.clone());
            save_datasets(&datasets);

            if *current_dataset == name {
                current_dataset.set(String::new());
                flashcards.set(Vec::new());
                known_cards.set(Vec::new());
                current_index.set(0);
                stage.set(FlashcardStage::First);
                exercise_answer.set(String::new());
                exercise_checked.set(false);
            }
        })
    };

    let on_start_rename = {
        let renaming_dataset = renaming_dataset.clone();
        let rename_input = rename_input.clone();
        Callback::from(move |name: String| {
            rename_input.set(name.clone());
            renaming_dataset.set(Some(name));
        })
    };

    let oninput_rename = {
        let rename_input = rename_input.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                rename_input.set(input.value());
            }
        })
    };

    let confirm_rename = {
        let renaming_dataset = renaming_dataset.clone();
        let rename_input = rename_input.clone();
        let datasets_list = datasets_list.clone();
        let current_dataset = current_dataset.clone();
        Callback::from(move |_: MouseEvent| {
            let new_name = (*rename_input).trim().to_string();
            if new_name.is_empty() {
                return;
            }
            let Some(old_name) = (*renaming_dataset).clone() else {
                return;
            };
            let mut datasets = (*datasets_list).clone();
            if new_name != old_name && datasets.iter().any(|d| d.name == new_name) {
                return;
            }
            if let Some(dataset) = datasets.iter_mut().find(|d| d.name == old_name) {
                dataset.name = new_name.clone();
            }
            if *current_dataset == old_name {
                current_dataset.set(new_name.clone());
            }
            datasets_list.set(datasets.clone());
            save_datasets(&datasets);
            renaming_dataset.set(None);
            rename_input.set(String::new());
        })
    };

    let cancel_rename = {
        let renaming_dataset = renaming_dataset.clone();
        let rename_input = rename_input.clone();
        Callback::from(move |_: MouseEvent| {
            renaming_dataset.set(None);
            rename_input.set(String::new());
        })
    };

    let on_file_select = {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let reader_handle = reader_handle.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |event: Event| {
            let Some(input) = event.target_dyn_into::<HtmlInputElement>() else {
                return;
            };

            let Some(files) = input.files() else {
                return;
            };

            let Some(file) = files.get(0) else {
                return;
            };

            let file = File::from(file);
            let flashcards = flashcards.clone();
            let known_cards = known_cards.clone();
            let reader_handle = reader_handle.clone();
            let exercise_answer = exercise_answer.clone();
            let exercise_checked = exercise_checked.clone();

            let task = gloo_file::callbacks::read_as_text(&file, move |result| {
                if let Ok(csv_data) = result {
                    let all_cards = parse_flashcards_from_csv(&csv_data);
                    let (known, unknown) = split_flashcards(all_cards);
                    flashcards.set(unknown);
                    known_cards.set(known);
                    exercise_answer.set(String::new());
                    exercise_checked.set(false);
                }
            });

            reader_handle.set(Some(task));
        })
    };

    let save_all_progress = {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        let direction = direction.clone();
        let current_dataset = current_dataset.clone();
        let datasets_list = datasets_list.clone();
        let backup_status_message = backup_status_message.clone();
        let backup_status_is_error = backup_status_is_error.clone();

        Callback::from(move |_: MouseEvent| {
            let persisted_state = PersistedState {
                flashcards: (*flashcards).clone(),
                known_cards: (*known_cards).clone(),
                current_index: *current_index,
                stage: *stage,
                direction: *direction,
                current_dataset: (*current_dataset).clone(),
            };
            let datasets = datasets_with_current_progress(
                &datasets_list,
                current_dataset.as_str(),
                &flashcards,
                &known_cards,
            );
            let snapshot = build_backup_snapshot(persisted_state, datasets);

            match serialize_backup_snapshot(&snapshot)
                .map_err(|_| "Could not build the backup file.".to_string())
                .and_then(|bytes| {
                    trigger_backup_download(&bytes, &backup_file_name())
                        .map_err(|_| "Could not start the backup download.".to_string())
                }) {
                Ok(()) => {
                    backup_status_is_error.set(false);
                    backup_status_message
                        .set(Some("Full backup downloaded successfully.".to_string()));
                }
                Err(error) => {
                    backup_status_is_error.set(true);
                    backup_status_message.set(Some(error));
                }
            }
        })
    };

    let open_backup_picker = {
        let backup_status_message = backup_status_message.clone();
        let backup_status_is_error = backup_status_is_error.clone();

        Callback::from(move |_: MouseEvent| {
            let Some(window) = web_sys::window() else {
                backup_status_is_error.set(true);
                backup_status_message.set(Some("Browser window is unavailable.".to_string()));
                return;
            };
            let Some(document) = window.document() else {
                backup_status_is_error.set(true);
                backup_status_message.set(Some("Browser document is unavailable.".to_string()));
                return;
            };
            let Some(element) = document.get_element_by_id("load-progress-input") else {
                backup_status_is_error.set(true);
                backup_status_message.set(Some("Load control is unavailable.".to_string()));
                return;
            };

            match element.dyn_into::<HtmlInputElement>() {
                Ok(input) => input.click(),
                Err(_) => {
                    backup_status_is_error.set(true);
                    backup_status_message
                        .set(Some("Load control could not be activated.".to_string()));
                }
            }
        })
    };

    let on_backup_file_select = {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        let direction = direction.clone();
        let current_dataset = current_dataset.clone();
        let datasets_list = datasets_list.clone();
        let reader_handle = reader_handle.clone();
        let backup_status_message = backup_status_message.clone();
        let backup_status_is_error = backup_status_is_error.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |event: Event| {
            let Some(input) = event.target_dyn_into::<HtmlInputElement>() else {
                backup_status_is_error.set(true);
                backup_status_message
                    .set(Some("Could not read the selected backup file.".to_string()));
                return;
            };

            let Some(files) = input.files() else {
                return;
            };

            let Some(file) = files.get(0) else {
                return;
            };

            let selected_file_name = file.name();
            input.set_value("");

            let file = File::from(file);
            let flashcards = flashcards.clone();
            let known_cards = known_cards.clone();
            let current_index = current_index.clone();
            let stage = stage.clone();
            let direction = direction.clone();
            let current_dataset = current_dataset.clone();
            let datasets_list = datasets_list.clone();
            let reader_handle = reader_handle.clone();
            let completion_reader_handle = reader_handle.clone();
            let backup_status_message = backup_status_message.clone();
            let backup_status_is_error = backup_status_is_error.clone();
            let exercise_answer = exercise_answer.clone();
            let exercise_checked = exercise_checked.clone();

            let task = gloo_file::callbacks::read_as_text(&file, move |result| {
                let parsed = result
                    .map_err(|_| "Could not read the backup file.".to_string())
                    .and_then(|json| parse_backup_snapshot(&json));

                match parsed {
                    Ok(snapshot) => {
                        let restored_state = normalized_restore_state(snapshot.persisted_state);
                        let restored_datasets = snapshot.datasets;

                        flashcards.set(restored_state.flashcards.clone());
                        known_cards.set(restored_state.known_cards.clone());
                        current_index.set(restored_state.current_index);
                        stage.set(restored_state.stage);
                        direction.set(restored_state.direction);
                        current_dataset.set(restored_state.current_dataset.clone());
                        datasets_list.set(restored_datasets.clone());
                        exercise_answer.set(String::new());
                        exercise_checked.set(false);

                        save_persisted_state(&restored_state);
                        save_datasets(&restored_datasets);

                        backup_status_is_error.set(false);
                        backup_status_message
                            .set(Some(format!("Backup loaded from {}.", selected_file_name)));
                    }
                    Err(error) => {
                        backup_status_is_error.set(true);
                        backup_status_message.set(Some(error));
                    }
                }

                completion_reader_handle.set(None);
            });

            reader_handle.set(Some(task));
        })
    };

    let on_card_click = {
        let stage = stage.clone();
        Callback::from(move |_: MouseEvent| {
            stage.set(match *stage {
                FlashcardStage::First => FlashcardStage::Second,
                FlashcardStage::Second => FlashcardStage::Third,
                FlashcardStage::Third => FlashcardStage::First,
            });
        })
    };

    let mark_known = {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |_: MouseEvent| {
            if flashcards.is_empty() {
                return;
            }

            let mut list = (*flashcards).clone();
            let mut card = list.remove(*current_index);
            card.known = true;

            let mut known = (*known_cards).clone();
            known.push(card);
            known_cards.set(known);

            if list.is_empty() {
                flashcards.set(Vec::new());
                current_index.set(0);
            } else {
                let new_index = if *current_index >= list.len() {
                    0
                } else {
                    *current_index
                };
                flashcards.set(list);
                current_index.set(new_index);
            }

            stage.set(FlashcardStage::First);
            exercise_answer.set(String::new());
            exercise_checked.set(false);
        })
    };

    let restore_card = {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |index: usize| {
            let mut known = (*known_cards).clone();
            if index < known.len() {
                let mut card = known.remove(index);
                card.known = false;

                let mut unknown = (*flashcards).clone();
                unknown.push(card);
                flashcards.set(unknown);
                known_cards.set(known);
                exercise_answer.set(String::new());
                exercise_checked.set(false);
            }
        })
    };

    let delete_flashcard = {
        let flashcards = flashcards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |_: MouseEvent| {
            if flashcards.is_empty() {
                return;
            }

            let mut list = (*flashcards).clone();
            list.remove(*current_index);

            if list.is_empty() {
                flashcards.set(Vec::new());
                current_index.set(0);
            } else {
                let new_index = if *current_index >= list.len() {
                    0
                } else {
                    *current_index
                };
                flashcards.set(list);
                current_index.set(new_index);
            }

            stage.set(FlashcardStage::First);
            exercise_answer.set(String::new());
            exercise_checked.set(false);
        })
    };

    let delete_known_card = {
        let known_cards = known_cards.clone();

        Callback::from(move |index: usize| {
            let mut known = (*known_cards).clone();
            if index < known.len() {
                known.remove(index);
                known_cards.set(known);
            }
        })
    };

    let on_toggle_unknown_in_table = {
        let show_unknown_in_table = show_unknown_in_table.clone();
        Callback::from(move |_: MouseEvent| {
            show_unknown_in_table.set(!*show_unknown_in_table);
        })
    };

    let mark_known_from_table = {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();
        Callback::from(move |index: usize| {
            let mut list = (*flashcards).clone();
            if index < list.len() {
                let mut card = list.remove(index);
                card.known = true;
                let mut known = (*known_cards).clone();
                known.push(card);
                known_cards.set(known);
                if list.is_empty() {
                    flashcards.set(Vec::new());
                    current_index.set(0);
                } else {
                    let new_idx = if *current_index >= list.len() {
                        0
                    } else {
                        *current_index
                    };
                    flashcards.set(list);
                    current_index.set(new_idx);
                }
                stage.set(FlashcardStage::First);
                exercise_answer.set(String::new());
                exercise_checked.set(false);
            }
        })
    };

    let delete_unknown_from_table = {
        let flashcards = flashcards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();
        Callback::from(move |index: usize| {
            let mut list = (*flashcards).clone();
            if index < list.len() {
                list.remove(index);
                if list.is_empty() {
                    flashcards.set(Vec::new());
                    current_index.set(0);
                } else {
                    let new_idx = if *current_index >= list.len() {
                        0
                    } else {
                        *current_index
                    };
                    flashcards.set(list);
                    current_index.set(new_idx);
                }
                stage.set(FlashcardStage::First);
                exercise_answer.set(String::new());
                exercise_checked.set(false);
            }
        })
    };

    let next_card = {
        let current_index = current_index.clone();
        let flashcards = flashcards.clone();
        let stage = stage.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |_: MouseEvent| {
            if !flashcards.is_empty() {
                current_index.set((*current_index + 1) % flashcards.len());
                stage.set(FlashcardStage::First);
                exercise_answer.set(String::new());
                exercise_checked.set(false);
            }
        })
    };

    let prev_card = {
        let current_index = current_index.clone();
        let flashcards = flashcards.clone();
        let stage = stage.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |_: MouseEvent| {
            if !flashcards.is_empty() {
                let prev = if *current_index == 0 {
                    flashcards.len() - 1
                } else {
                    *current_index - 1
                };
                current_index.set(prev);
                stage.set(FlashcardStage::First);
                exercise_answer.set(String::new());
                exercise_checked.set(false);
            }
        })
    };

    let toggle_direction = {
        let direction = direction.clone();
        let stage = stage.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |_: MouseEvent| {
            direction.set(match *direction {
                StudyDirection::Normal => StudyDirection::Reverse,
                StudyDirection::Reverse => StudyDirection::Normal,
            });
            stage.set(FlashcardStage::First);
            exercise_answer.set(String::new());
            exercise_checked.set(false);
        })
    };

    let toggle_study_mode = {
        let study_mode = study_mode.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |_: MouseEvent| {
            study_mode.set(match *study_mode {
                StudyMode::Reveal => StudyMode::Exercise,
                StudyMode::Exercise => StudyMode::Reveal,
            });
            exercise_answer.set(String::new());
            exercise_checked.set(false);
        })
    };

    let randomize_cards = {
        let flashcards = flashcards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |_: MouseEvent| {
            let mut shuffled = (*flashcards).clone();
            let mut rng = thread_rng();
            shuffled.shuffle(&mut rng);
            flashcards.set(shuffled);
            current_index.set(0);
            stage.set(FlashcardStage::First);
            exercise_answer.set(String::new());
            exercise_checked.set(false);
        })
    };

    let oninput_exercise_answer = {
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                exercise_answer.set(input.value());
                exercise_checked.set(false);
            }
        })
    };

    let check_exercise_answer = {
        let flashcards = flashcards.clone();
        let current_index = current_index.clone();
        let direction = direction.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();
        let exercise_is_correct = exercise_is_correct.clone();

        Callback::from(move |_| {
            let Some(card) = flashcards.get(*current_index) else {
                return;
            };
            let expected = exercise_expected_answer(card, *direction);
            let is_correct = is_exercise_answer_correct(&exercise_answer, &expected, *direction);

            exercise_is_correct.set(is_correct);
            exercise_checked.set(true);
        })
    };

    let on_exercise_keydown = {
        let check_exercise_answer = check_exercise_answer.clone();

        Callback::from(move |event: KeyboardEvent| {
            if event.key() == "Enter" {
                event.prevent_default();
                check_exercise_answer.emit(());
            }
        })
    };

    let open_add = {
        let show_add = show_add.clone();
        Callback::from(move |_: MouseEvent| show_add.set(true))
    };

    let open_help = {
        let show_help = show_help.clone();
        Callback::from(move |_: MouseEvent| show_help.set(true))
    };

    let close_help = {
        let show_help = show_help.clone();
        Callback::from(move |_: MouseEvent| show_help.set(false))
    };

    let close_add = {
        let show_add = show_add.clone();
        Callback::from(move |_: MouseEvent| show_add.set(false))
    };

    let oninput_new_word = {
        let new_word = new_word.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                new_word.set(input.value());
            }
        })
    };

    let oninput_new_pinyin = {
        let new_pinyin = new_pinyin.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                new_pinyin.set(input.value());
            }
        })
    };

    let oninput_new_translation = {
        let new_translation = new_translation.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                new_translation.set(input.value());
            }
        })
    };

    let save_new = {
        let flashcards = flashcards.clone();
        let new_word = new_word.clone();
        let new_pinyin = new_pinyin.clone();
        let new_translation = new_translation.clone();
        let show_add = show_add.clone();
        let exercise_answer = exercise_answer.clone();
        let exercise_checked = exercise_checked.clone();

        Callback::from(move |_: MouseEvent| {
            let mut list = (*flashcards).clone();
            let pinyin = if new_pinyin.is_empty() {
                None
            } else {
                Some((*new_pinyin).clone())
            };

            list.push(Flashcard {
                word: (*new_word).clone(),
                pinyin,
                translation: (*new_translation).clone(),
                known: false,
            });

            flashcards.set(list);
            new_word.set(String::new());
            new_pinyin.set(String::new());
            new_translation.set(String::new());
            show_add.set(false);
            exercise_answer.set(String::new());
            exercise_checked.set(false);
        })
    };

    let update_information = {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();

        Callback::from(move |_| {
            if let Ok(bytes) = export_flashcards_csv(flashcards.iter().chain(known_cards.iter())) {
                let _ = trigger_csv_download(&bytes, "updated_flashcards.csv");
            }
        })
    };

    let show_export = !current_dataset.is_empty();
    let show_import = show_export && flashcards.is_empty() && known_cards.is_empty();

    let known_total = flashcards.len() + known_cards.len();

    let position_counter = if !flashcards.is_empty() {
        html! {
            <p class="position-counter">
                { format!("Unknown card: {} / {}", *current_index + 1, flashcards.len()) }
            </p>
        }
    } else {
        html! {}
    };

    let current_card = flashcards.get(*current_index);
    let current_card_text = current_card.map(|card| match *study_mode {
        StudyMode::Reveal => display_text(card, *direction, *stage),
        StudyMode::Exercise => exercise_prompt(card, *direction),
    });

    let expected_answer = current_card.map(|card| exercise_expected_answer(card, *direction));

    let exercise_feedback = if *study_mode == StudyMode::Exercise && *exercise_checked {
        if *exercise_is_correct {
            Some("Correct!".to_string())
        } else {
            expected_answer
                .as_ref()
                .map(|answer| format!("Incorrect. Correct answer: {}", answer))
        }
    } else {
        None
    };

    let exercise_feedback_class = if *study_mode == StudyMode::Exercise && *exercise_checked {
        Some(if *exercise_is_correct {
            "is-correct"
        } else {
            "is-incorrect"
        })
    } else {
        None
    };

    let exercise_pinyin = if *study_mode == StudyMode::Exercise && *exercise_checked {
        current_card.and_then(|card| card.pinyin.clone())
    } else {
        None
    };

    html! {
        <div class="app-shell">
            <header class="app-header">
                <h1 class="app-title">{"Language Flashcards 🈶"}</h1>
                <p class="app-subtitle">{"Build your vocabulary with cleaner practice sessions and smarter review flow."}</p>
                <button class="btn btn-secondary btn-small help-trigger-btn" onclick={open_help}>
                    {"? Help & About"}
                </button>
            </header>

            if *show_help {
                <HelpPanel on_close={close_help} />
            }

            <DatasetPanel
                datasets={(*datasets_list).clone()}
                current_dataset={(*current_dataset).clone()}
                show_dataset_input={*show_dataset_input}
                new_dataset_name={(*new_dataset_name).clone()}
                on_select_dataset={load_dataset.clone()}
                on_delete_dataset={delete_dataset.clone()}
                on_toggle_input={{
                    let show_dataset_input = show_dataset_input.clone();
                    Callback::from(move |_: MouseEvent| show_dataset_input.set(!*show_dataset_input))
                }}
                on_dataset_name_input={oninput_dataset_name.clone()}
                on_create_dataset={add_new_dataset.clone()}
                on_save_all={save_all_progress.clone()}
                on_open_backup_picker={open_backup_picker.clone()}
                on_backup_file_select={on_backup_file_select.clone()}
                backup_status_message={(*backup_status_message).clone()}
                backup_status_is_error={*backup_status_is_error}
                show_import={show_import}
                show_export={show_export}
                on_file_select={on_file_select.clone()}
                on_download={update_information.clone()}
                renaming_dataset={(*renaming_dataset).clone()}
                rename_input={(*rename_input).clone()}
                on_start_rename={on_start_rename.clone()}
                on_rename_input={oninput_rename.clone()}
                on_confirm_rename={confirm_rename.clone()}
                on_cancel_rename={cancel_rename.clone()}
            />

            <StudyToolbar
                direction={*direction}
                mode={*study_mode}
                on_toggle_mode={toggle_study_mode.clone()}
                on_toggle_direction={toggle_direction.clone()}
                on_randomize={randomize_cards.clone()}
                on_open_add={open_add.clone()}
            />

            <AddFlashcardForm
                visible={*show_add}
                new_word={(*new_word).clone()}
                new_pinyin={(*new_pinyin).clone()}
                new_translation={(*new_translation).clone()}
                on_word_input={oninput_new_word.clone()}
                on_pinyin_input={oninput_new_pinyin.clone()}
                on_translation_input={oninput_new_translation.clone()}
                on_save={save_new.clone()}
                on_cancel={close_add.clone()}
            />

            <section class="unknown-panel panel">
                <h3 class="panel-title">{"Flashcards"}</h3>
                <div class="unknown-meta">
                    { position_counter }
                </div>

                <FlashcardView
                    mode={*study_mode}
                    card_text={current_card_text}
                    card_state_class={exercise_feedback_class}
                    on_card_click={on_card_click.clone()}
                    exercise_answer={(*exercise_answer).clone()}
                    on_exercise_input={oninput_exercise_answer.clone()}
                    on_exercise_keydown={on_exercise_keydown.clone()}
                    on_check_answer={check_exercise_answer.clone()}
                    exercise_feedback={exercise_feedback}
                    exercise_feedback_class={exercise_feedback_class}
                    exercise_pinyin={exercise_pinyin}
                    on_prev={prev_card.clone()}
                    on_mark_known={mark_known.clone()}
                    on_delete={delete_flashcard.clone()}
                    on_next={next_card.clone()}
                />
            </section>

            <KnownCardsTable
                known_cards={(*known_cards).clone()}
                unknown_cards={(*flashcards).clone()}
                total={known_total}
                show_unknown={*show_unknown_in_table}
                on_restore={restore_card.clone()}
                on_delete={delete_known_card.clone()}
                on_toggle_unknown={on_toggle_unknown_in_table.clone()}
                on_mark_known_from_table={mark_known_from_table.clone()}
                on_delete_unknown={delete_unknown_from_table.clone()}
            />

            <footer class="app-footer">
                {"Companion vocabulary tool for learners using "}
                <a href="https://chinesewithbaiba.eu" target="_blank" rel="noopener noreferrer">{"chinesewithbaiba.eu"}</a>
            </footer>
        </div>
    }
}
