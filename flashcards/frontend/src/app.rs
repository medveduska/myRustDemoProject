use gloo_file::callbacks::FileReader;
use gloo_file::File;
use rand::seq::SliceRandom;
use rand::thread_rng;
use web_sys::{HtmlInputElement, InputEvent, MouseEvent};
use yew::prelude::*;

use crate::components::add_flashcard_form::AddFlashcardForm;
use crate::components::dataset_panel::DatasetPanel;
use crate::components::flashcard_view::FlashcardView;
use crate::components::help_panel::HelpPanel;
use crate::components::known_cards_table::KnownCardsTable;
use crate::components::study_toolbar::StudyToolbar;
use crate::csv_io::{export_flashcards_csv, parse_flashcards_from_csv, trigger_csv_download};
use crate::model::{Dataset, Flashcard, FlashcardStage, PersistedState, StudyDirection};
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
    let current_dataset = use_state(String::new);
    let datasets_list = use_state(move || datasets);
    let new_dataset_name = use_state(String::new);
    let show_dataset_input = use_state(|| false);
    let show_add = use_state(|| false);
    let show_help = use_state(|| false);
    let new_word = use_state(String::new);
    let new_pinyin = use_state(String::new);
    let new_translation = use_state(String::new);

    {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        let direction = direction.clone();

        use_effect_with(
            (
                flashcards.clone(),
                known_cards.clone(),
                current_index.clone(),
                stage.clone(),
                direction.clone(),
            ),
            move |_| {
                save_persisted_state(&PersistedState {
                    flashcards: (*flashcards).clone(),
                    known_cards: (*known_cards).clone(),
                    current_index: *current_index,
                    stage: *stage,
                    direction: *direction,
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

        Callback::from(move |name: String| {
            if let Some(dataset) = datasets_list.iter().find(|dataset| dataset.name == name) {
                flashcards.set(dataset.flashcards.clone());
                known_cards.set(dataset.known_cards.clone());
                current_index.set(0);
                stage.set(FlashcardStage::First);
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
            }
        })
    };

    let on_file_select = {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let reader_handle = reader_handle.clone();

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

            let task = gloo_file::callbacks::read_as_text(&file, move |result| {
                if let Ok(csv_data) = result {
                    let all_cards = parse_flashcards_from_csv(&csv_data);
                    let (known, unknown) = split_flashcards(all_cards);
                    flashcards.set(unknown);
                    known_cards.set(known);
                }
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
        })
    };

    let restore_card = {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();

        Callback::from(move |index: usize| {
            let mut known = (*known_cards).clone();
            if index < known.len() {
                let mut card = known.remove(index);
                card.known = false;

                let mut unknown = (*flashcards).clone();
                unknown.push(card);
                flashcards.set(unknown);
                known_cards.set(known);
            }
        })
    };

    let delete_flashcard = {
        let flashcards = flashcards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();

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

    let next_card = {
        let current_index = current_index.clone();
        let flashcards = flashcards.clone();
        let stage = stage.clone();

        Callback::from(move |_: MouseEvent| {
            if !flashcards.is_empty() {
                current_index.set((*current_index + 1) % flashcards.len());
                stage.set(FlashcardStage::First);
            }
        })
    };

    let prev_card = {
        let current_index = current_index.clone();
        let flashcards = flashcards.clone();
        let stage = stage.clone();

        Callback::from(move |_: MouseEvent| {
            if !flashcards.is_empty() {
                let prev = if *current_index == 0 {
                    flashcards.len() - 1
                } else {
                    *current_index - 1
                };
                current_index.set(prev);
                stage.set(FlashcardStage::First);
            }
        })
    };

    let toggle_direction = {
        let direction = direction.clone();
        let stage = stage.clone();

        Callback::from(move |_: MouseEvent| {
            direction.set(match *direction {
                StudyDirection::Normal => StudyDirection::Reverse,
                StudyDirection::Reverse => StudyDirection::Normal,
            });
            stage.set(FlashcardStage::First);
        })
    };

    let randomize_cards = {
        let flashcards = flashcards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();

        Callback::from(move |_: MouseEvent| {
            let mut shuffled = (*flashcards).clone();
            let mut rng = thread_rng();
            shuffled.shuffle(&mut rng);
            flashcards.set(shuffled);
            current_index.set(0);
            stage.set(FlashcardStage::First);
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

    let current_card_text = flashcards
        .get(*current_index)
        .map(|card| display_text(card, *direction, *stage));

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
                show_import={show_import}
                show_export={show_export}
                on_file_select={on_file_select.clone()}
                on_download={update_information.clone()}
            />

            <StudyToolbar
                direction={*direction}
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
                <h3 class="panel-title">{"Unknown Words"}</h3>
                <div class="unknown-meta">
                    { position_counter }
                </div>

                <FlashcardView
                    card_text={current_card_text}
                    on_card_click={on_card_click.clone()}
                    on_prev={prev_card.clone()}
                    on_mark_known={mark_known.clone()}
                    on_delete={delete_flashcard.clone()}
                    on_next={next_card.clone()}
                />
            </section>

            <KnownCardsTable
                known_cards={(*known_cards).clone()}
                total={known_total}
                on_restore={restore_card.clone()}
                on_delete={delete_known_card.clone()}
            />

            <footer class="app-footer">
                {"Companion vocabulary tool for learners using "}
                <a href="https://chinesewithbaiba.eu" target="_blank" rel="noopener noreferrer">{"chinesewithbaiba.eu"}</a>
            </footer>
        </div>
    }
}
