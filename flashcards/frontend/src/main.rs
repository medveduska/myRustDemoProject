use yew::prelude::*;
use serde::{Deserialize, Serialize};
use gloo_file::callbacks::FileReader;
use gloo_file::File;
use web_sys::{HtmlInputElement, Blob, Url, InputEvent, MouseEvent};
use wasm_bindgen::JsCast;
use js_sys::{Uint8Array, Array};
use rand::seq::SliceRandom;
use rand::thread_rng;
use gloo_storage::{LocalStorage, Storage};

const STORAGE_KEY: &str = "flashcards_app_state";
const DATASETS_KEY: &str = "flashcards_datasets_list";

#[derive(Deserialize, Serialize, Clone, PartialEq)]
struct Flashcard {
    word: String,
    pinyin: Option<String>,
    translation: String,
    #[serde(default)]
    known: bool,
}

#[derive(Deserialize, Serialize, Clone)]
struct Dataset {
    name: String,
    flashcards: Vec<Flashcard>,
    known_cards: Vec<Flashcard>,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum FlashcardStage {
    First,
    Second,
    Third,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum StudyDirection {
    Normal,
    Reverse,
}

#[derive(Serialize, Deserialize)]
struct PersistedState {
    flashcards: Vec<Flashcard>,
    known_cards: Vec<Flashcard>,
    current_index: usize,
    stage: FlashcardStage,
    direction: StudyDirection,
}


#[function_component(App)]
fn app() -> Html {
    let persisted: Option<PersistedState> = LocalStorage::get(STORAGE_KEY).ok();

    let flashcards = use_state(|| persisted.as_ref().map(|p| p.flashcards.clone()).unwrap_or_default());
    let known_cards = use_state(|| persisted.as_ref().map(|p| p.known_cards.clone()).unwrap_or_default());
    let current_index = use_state(|| persisted.as_ref().map(|p| p.current_index).unwrap_or(0));
    let stage = use_state(|| persisted.as_ref().map(|p| p.stage).unwrap_or(FlashcardStage::First));
    let direction = use_state(|| persisted.as_ref().map(|p| p.direction).unwrap_or(StudyDirection::Normal));
    let _reader_handle = use_state(|| None::<FileReader>);
    
    // Dataset management
    let datasets: Vec<Dataset> = LocalStorage::get(DATASETS_KEY).ok().unwrap_or_default();
    let current_dataset = use_state(|| String::new());
    let datasets_list = use_state(move || datasets);
    let new_dataset_name = use_state(|| String::new());
    let show_dataset_input = use_state(|| false);
    // Auto-save to local storage whenever state changes
    {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        let direction = direction.clone();

        use_effect_with(
            (flashcards.clone(), known_cards.clone(), current_index.clone(), stage.clone(), direction.clone()),
            move |_| {
                let state = PersistedState {
                    flashcards: (*flashcards).clone(),
                    known_cards: (*known_cards).clone(),
                    current_index: *current_index,
                    stage: *stage,
                    direction: *direction,
                };
                let _ = LocalStorage::set(STORAGE_KEY, state);
                || ()
            },
        );
    }
    
    // Load dataset when selected
    let load_dataset = {
        let datasets_list = datasets_list.clone();
        let current_dataset = current_dataset.clone();
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        
        Callback::from(move |name: String| {
            if let Some(dataset) = datasets_list.iter().find(|d| d.name == name) {
                flashcards.set(dataset.flashcards.clone());
                known_cards.set(dataset.known_cards.clone());
                current_index.set(0);
                stage.set(FlashcardStage::First);
                current_dataset.set(name);
            }
        })
    };
    
    // Autosave current dataset whenever flashcards or known_cards change
    {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_dataset = current_dataset.clone();
        let datasets_list = datasets_list.clone();
        
        use_effect_with(
            (flashcards.clone(), known_cards.clone(), current_dataset.clone()),
            move |_| {
                if !current_dataset.is_empty() {
                    let mut datasets = (*datasets_list).clone();
                    if let Some(dataset) = datasets.iter_mut().find(|d| d.name == *current_dataset) {
                        dataset.flashcards = (*flashcards).clone();
                        dataset.known_cards = (*known_cards).clone();
                        datasets_list.set(datasets.clone());
                        let _ = LocalStorage::set(DATASETS_KEY, datasets);
                    }
                }
                || ()
            },
        );
    }
    
    // Add new dataset
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
                if !datasets.iter().any(|d| d.name == *new_dataset_name) {
                    datasets.push(Dataset {
                        name: (*new_dataset_name).clone(),
                        flashcards: vec![],
                        known_cards: vec![],
                    });
                    datasets_list.set(datasets.clone());
                    current_dataset.set((*new_dataset_name).clone());
                    // Clear the working state for the new empty dataset
                    flashcards.set(vec![]);
                    known_cards.set(vec![]);
                    current_index.set(0);
                    stage.set(FlashcardStage::First);
                    let _ = LocalStorage::set(DATASETS_KEY, datasets);
                }
                new_dataset_name.set(String::new());
                show_dataset_input.set(false);
            }
        })
    };
    
    // Input handler for new dataset name
    let oninput_dataset_name = {
        let new_dataset_name = new_dataset_name.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                new_dataset_name.set(input.value());
            }
        })
    };
    
    // Delete dataset
    let delete_dataset = {
        let datasets_list = datasets_list.clone();
        let current_dataset = current_dataset.clone();
        
        Callback::from(move |name: String| {
            let mut datasets = (*datasets_list).clone();
            datasets.retain(|d| d.name != name);
            datasets_list.set(datasets.clone());
            let _ = LocalStorage::set(DATASETS_KEY, datasets);
            
            if *current_dataset == name {
                current_dataset.set(String::new());
            }
        })
    };
    
    // ---------- File selection ----------
    let on_file_select = {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let reader_handle = _reader_handle.clone();

        Callback::from(move |event: Event| {
            let input: HtmlInputElement = event.target_dyn_into().unwrap();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let file = File::from(file);
                    let flashcards = flashcards.clone();
                    let known_cards = known_cards.clone();
                    let reader_handle = reader_handle.clone();

                    let task = gloo_file::callbacks::read_as_text(&file, move |res| {
                        if let Ok(csv_data) = res {
                            let mut rdr = csv::ReaderBuilder::new()
                                .has_headers(false)
                                .from_reader(csv_data.as_bytes());
                            let mut all = Vec::new();

                            for record in rdr.records() {
                                if let Ok(r) = record {
                                    let word = r.get(0).unwrap_or("").to_string();
                                    let pinyin = r.get(1).map(|s| s.to_string());
                                    let translation = r.get(2).unwrap_or("").to_string();
                                    let known = r
                                        .get(3)
                                        .map(|v| v.trim().eq_ignore_ascii_case("true"))
                                        .unwrap_or(false);
                                    all.push(Flashcard {
                                        word,
                                        pinyin,
                                        translation,
                                        known,
                                    });
                                }
                            }

                            let (known, unknown): (Vec<_>, Vec<_>) =
                                all.into_iter().partition(|c| c.known);

                            flashcards.set(unknown);
                            known_cards.set(known);
                        }
                    });
                    reader_handle.set(Some(task));
                }
            }
        })
    };

    // ---------- Card click cycle ----------
    let on_card_click = {
        let stage = stage.clone();
        Callback::from(move |_| {
            stage.set(match *stage {
                FlashcardStage::First => FlashcardStage::Second,
                FlashcardStage::Second => FlashcardStage::Third,
                FlashcardStage::Third => FlashcardStage::First,
            });
        })
    };

    // ---------- Mark as Known ----------
    let mark_known = {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();

        Callback::from(move |_| {
            if !flashcards.is_empty() {
                let mut list = (*flashcards).clone();
                let mut card = list.remove(*current_index);
                card.known = true;

                let mut known = (*known_cards).clone();
                known.push(card);
                known_cards.set(known);

                if list.is_empty() {
                    flashcards.set(vec![]);
                    current_index.set(0);
                } else {
                    let new_idx = if *current_index >= list.len() { 0 } else { *current_index };
                    flashcards.set(list);
                    current_index.set(new_idx);
                }
                stage.set(FlashcardStage::First);
            }
        })
    };

    // ---------- Restore known ----------
    let restore_card = {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();
        Callback::from(move |index: usize| {
            let mut known = (*known_cards).clone();
            if index < known.len() {
                let mut card = known.remove(index);
                card.known = false;
                let mut flash = (*flashcards).clone();
                flash.push(card);
                flashcards.set(flash);
                known_cards.set(known);
            }
        })
    };
    
    // ---------- Delete unknown flashcard ----------
    let delete_flashcard = {
        let flashcards = flashcards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();
        
        Callback::from(move |_: MouseEvent| {
            if !flashcards.is_empty() {
                let mut list = (*flashcards).clone();
                list.remove(*current_index);
                
                if list.is_empty() {
                    flashcards.set(vec![]);
                    current_index.set(0);
                } else {
                    let new_idx = if *current_index >= list.len() { 0 } else { *current_index };
                    flashcards.set(list);
                    current_index.set(new_idx);
                }
                stage.set(FlashcardStage::First);
            }
        })
    };
    
    // ---------- Delete known flashcard ----------
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

    // ---------- Navigation ----------
    let next_card = {
        let current_index = current_index.clone();
        let flashcards = flashcards.clone();
        let stage = stage.clone();
        Callback::from(move |_| {
            if !flashcards.is_empty() {
                let next = (*current_index + 1) % flashcards.len();
                current_index.set(next);
                stage.set(FlashcardStage::First);
            }
        })
    };

    let prev_card = {
        let current_index = current_index.clone();
        let flashcards = flashcards.clone();
        let stage = stage.clone();
        Callback::from(move |_| {
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

    // ---------- Direction toggle ----------
    let toggle_direction = {
        let direction = direction.clone();
        let stage = stage.clone();
        Callback::from(move |_| {
            direction.set(match *direction {
                StudyDirection::Normal => StudyDirection::Reverse,
                StudyDirection::Reverse => StudyDirection::Normal,
            });
            stage.set(FlashcardStage::First);
        })
    };

    // ---------- Randomize unknown cards ----------
    let randomize_cards = {
        let flashcards = flashcards.clone();
        let current_index = current_index.clone();
        let stage = stage.clone();

        Callback::from(move |_| {
            let mut shuffled = (*flashcards).clone();
            let mut rng = thread_rng();
            shuffled.shuffle(&mut rng);
            flashcards.set(shuffled);
            current_index.set(0);
            stage.set(FlashcardStage::First);
        })
    };

    // ---------- Add new flashcard page/state ----------
    let show_add = use_state(|| false);
    let new_word = use_state(|| String::new());
    let new_pinyin = use_state(|| String::new());
    let new_translation = use_state(|| String::new());

    let open_add = {
        let show_add = show_add.clone();
        Callback::from(move |_| show_add.set(true))
    };

    let close_add = {
        let show_add = show_add.clone();
        Callback::from(move |_| show_add.set(false))
    };

    let oninput_new_word = {
        let new_word = new_word.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                new_word.set(input.value());
            }
        })
    };

    let oninput_new_pinyin = {
        let new_pinyin = new_pinyin.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                new_pinyin.set(input.value());
            }
        })
    };

    let oninput_new_translation = {
        let new_translation = new_translation.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
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

        Callback::from(move |_| {
            let mut list = (*flashcards).clone();
            let pinyin_opt = if new_pinyin.is_empty() { None } else { Some((*new_pinyin).clone()) };

            list.push(Flashcard {
                word: (*new_word).clone(),
                pinyin: pinyin_opt,
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

    // ---------- Export updated CSV ----------
    let update_information = {
        let flashcards = flashcards.clone();
        let known_cards = known_cards.clone();

        Callback::from(move |_| {
            let mut wtr = csv::Writer::from_writer(vec![]);
            for c in flashcards.iter().chain(known_cards.iter()) {
                let _ = wtr.write_record(&[
                    c.word.as_str(),
                    c.pinyin.as_deref().unwrap_or(""),
                    c.translation.as_str(),
                    if c.known { "true" } else { "false" },
                ]);
            }

            if let Ok(bytes) = wtr.into_inner() {
                let array = Uint8Array::from(&bytes[..]);
                let blob_parts = Array::new();
                blob_parts.push(&array.buffer());
                let blob = Blob::new_with_u8_array_sequence(&blob_parts).unwrap();
                let url = Url::create_object_url_with_blob(&blob).unwrap();

                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let a = document.create_element("a").unwrap();
                a.set_attribute("href", &url).unwrap();
                a.set_attribute("download", "updated_flashcards.csv").unwrap();
                let a: web_sys::HtmlElement = a.dyn_into().unwrap();
                a.click();
                Url::revoke_object_url(&url).unwrap();
            }
        })
    };

    // ---------- Progress counter ----------
    let progress_bar = {
        let total = flashcards.len() + known_cards.len();
        if total > 0 {
            html! {
                <p style="margin-top:10px; font-weight:bold;">
                    { format!("Known: {} / {}", known_cards.len(), total) }
                </p>
            }
        } else {
            html! {}
        }
    };

    let position_counter = if !flashcards.is_empty() {
        html! {
            <p style="margin-top:5px; color:#555;">
                { format!("Unknown card: {} / {}", *current_index + 1, flashcards.len()) }
            </p>
        }
    } else {
        html! {}
    };
    

    // ---------- Determine card display ----------
    let content = if !flashcards.is_empty() {
        let card = &flashcards[*current_index];
        let display_text = match (*direction, *stage) {
            (StudyDirection::Normal, FlashcardStage::First) => card.word.clone(),
            (StudyDirection::Normal, FlashcardStage::Second) => card.pinyin.clone().unwrap_or_default(),
            (StudyDirection::Normal, FlashcardStage::Third) => card.translation.clone(),
            (StudyDirection::Reverse, FlashcardStage::First) => card.translation.clone(),
            (StudyDirection::Reverse, FlashcardStage::Second) => card.pinyin.clone().unwrap_or_default(),
            (StudyDirection::Reverse, FlashcardStage::Third) => card.word.clone(),
        };

        html! {
            <>
                <div
                    onclick={on_card_click}
                    style="
                        margin: 50px auto;
                        width: 280px;
                        height: 180px;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        border-radius: 12px;
                        box-shadow: 0 4px 8px rgba(0,0,0,0.2);
                        font-size: 32px;
                        cursor: pointer;
                        user-select: none;
                    "
                >
                    { display_text }
                </div>

                <div style="margin-top: 10px;">
                    <button onclick={prev_card.clone()}>{"← Prev"}</button>
                    <button onclick={mark_known.clone()} style="margin: 0 10px;">{"Mark as Known ✅"}</button>
                    <button onclick={delete_flashcard.clone()} style="margin: 0 10px; background-color: #ffebee; color: #d32f2f;">{"🗑 Delete"}</button>
                    <button onclick={next_card.clone()}>{"Next →"}</button>
                </div>
            </>
        }
    } else {
        html! { <p>{"No unknown flashcards remaining."}</p> }
    };

    // ---------- Render ----------
    html! {
        <div style="font-family: sans-serif; text-align: center; margin-top: 40px;">
            <h1>{"Language Flashcards 🈶"}</h1>

            // Dataset management section
            <div style="margin-bottom: 20px; padding: 12px; background-color: #f5f5f5; border-radius: 8px;">
                <h3 style="margin-top: 0;">{"📚 Datasets"}</h3>
                <div style="margin-bottom: 10px;">
                    { if !datasets_list.is_empty() {
                        html! {
                            <div>
                                { for datasets_list.iter().map(|dataset| {
                                    let load_dataset = load_dataset.clone();
                                    let delete_dataset = delete_dataset.clone();
                                    let name = dataset.name.clone();
                                    let name_for_delete = dataset.name.clone();
                                    let is_selected = *current_dataset == dataset.name;
                                    html! {
                                        <div key={dataset.name.clone()} style="display: inline-block; margin: 4px;">
                                            <button 
                                                onclick={Callback::from(move |_| {
                                                    load_dataset.emit(name.clone());
                                                })}
                                                style={format!(
                                                    "padding: 8px 12px; border-radius: 4px 0 0 4px; border: 2px solid {}; background-color: {}; cursor: pointer; font-weight: {};",
                                                    if is_selected { "#2196F3" } else { "#ccc" },
                                                    if is_selected { "#e3f2fd" } else { "white" },
                                                    if is_selected { "bold" } else { "normal" }
                                                )}
                                            >
                                                { &dataset.name }
                                            </button>
                                            <button 
                                                onclick={Callback::from(move |_| {
                                                    delete_dataset.emit(name_for_delete.clone());
                                                })}
                                                style="padding: 8px 8px; border-radius: 0 4px 4px 0; border: 2px solid #ccc; background-color: #ffebee; cursor: pointer; color: #d32f2f; font-weight: bold; margin-left: -2px;"
                                                title="Delete this dataset"
                                            >
                                                { "🗑" }
                                            </button>
                                        </div>
                                    }
                                }) }
                            </div>
                        }
                    } else {
                        html! { <p style="color: #999;">{"No datasets yet. Create one below."}</p> }
                    } }
                </div>
                <div style="margin-bottom: 10px;">
                    <button onclick={{
                        let show_dataset_input = show_dataset_input.clone();
                        Callback::from(move |_| show_dataset_input.set(!*show_dataset_input))
                    }}>
                        { if *show_dataset_input { "✖ Cancel" } else { "➕ New Dataset" } }
                    </button>
                </div>
                { if *show_dataset_input {
                    html! {
                        <div style="margin-top: 10px; text-align: center;">
                            <input 
                                type="text"
                                placeholder="Dataset name (e.g., 'HSK 1', 'Business Terms')"
                                value={(*new_dataset_name).clone()}
                                oninput={oninput_dataset_name.clone()}
                                style="padding: 8px; width: 250px; margin-right: 8px; border-radius: 4px; border: 1px solid #ccc;"
                            />
                            <button onclick={add_new_dataset.clone()}>{"Create"}</button>
                        </div>
                    }
                } else {
                    html! {}
                } }
            </div>

            // File management section
            <div style="margin: 20px 0; padding: 12px; background-color: #f5f5f5; border-radius: 8px;">
                <h3 style="margin-top: 0;">{"📁 File Management"}</h3>
                <div style="margin-bottom: 12px;">
                    <input type="file" accept=".csv" onchange={on_file_select}/>
                </div>
                <button onclick={update_information.clone()}>
                    {"⬇️ Download Flashcards"}
                </button>
            </div>

            <div style="margin-top: 15px;">
                <button onclick={toggle_direction.clone()}>
                    {
                        match *direction {
                            StudyDirection::Normal => "Switch to Translation → Pinyin → Character",
                            StudyDirection::Reverse => "Switch to Character → Pinyin → Translation",
                        }
                    }
                </button>
                <button onclick={randomize_cards.clone()} style="margin-left: 10px;">
                    {"🔀 Randomize"}
                </button>
                <button onclick={open_add.clone()} style="margin-left: 10px;">{"➕ Add New Flashcard"}</button>
            </div>

            { if *show_add {
                html!{
                    <div style="margin-top:20px; padding:12px; border:1px solid #ddd; display:inline-block; text-align:left; border-radius:8px;">
                        <h3 style="margin:0 0 8px 0;">{"Add New Flashcard"}</h3>
                        <div style="margin-bottom:8px;"><input placeholder="Chinese character" value={(*new_word).clone()} oninput={oninput_new_word.clone()} /></div>
                        <div style="margin-bottom:8px;"><input placeholder="Pinyin" value={(*new_pinyin).clone()} oninput={oninput_new_pinyin.clone()} /></div>
                        <div style="margin-bottom:8px;"><input placeholder="Translation" value={(*new_translation).clone()} oninput={oninput_new_translation.clone()} /></div>
                        <div>
                            <button onclick={save_new.clone()}>{"Save"}</button>
                            <button onclick={close_add.clone()} style="margin-left:8px;">{"Cancel"}</button>
                        </div>
                    </div>
                }
            } else { html!{} } }

            { progress_bar }
            { position_counter }
            { content }

            <h3 style="margin-top: 40px;">{"Known Words"}</h3>
            <table style="margin: 0 auto; border-collapse: collapse;">
                <tr>
                    <th style="padding: 5px; border-bottom: 1px solid #ccc;">{"Word"}</th>
                    <th style="padding: 5px; border-bottom: 1px solid #ccc;">{"Pinyin"}</th>
                    <th style="padding: 5px; border-bottom: 1px solid #ccc;">{"Translation"}</th>
                    <th style="padding: 5px; border-bottom: 1px solid #ccc;">{"Action"}</th>
                </tr>
                { for known_cards.iter().enumerate().map(|(i, c)| html!{
                    <tr>
                        <td style="padding: 5px;">{ &c.word }</td>
                        <td style="padding: 5px;">{ c.pinyin.as_deref().unwrap_or("") }</td>
                        <td style="padding: 5px;">{ &c.translation }</td>
                        <td style="padding: 5px;">
                            <button onclick={{
                                let restore_card = restore_card.clone();
                                Callback::from(move |_| restore_card.emit(i))
                            }}>{"↩ Restore"}</button>
                            <button onclick={{
                                let delete_known_card = delete_known_card.clone();
                                Callback::from(move |_| delete_known_card.emit(i))
                            }} style="margin-left: 5px; background-color: #ffebee; color: #d32f2f;">{"🗑 Delete"}</button>
                        </td>
                    </tr>
                }) }
            </table>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
