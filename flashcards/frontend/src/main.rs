use yew::prelude::*;
use serde::{Deserialize, Serialize};
use gloo_file::callbacks::FileReader;
use gloo_file::File;
use web_sys::{HtmlInputElement, Blob, Url};
use wasm_bindgen::JsCast;
use js_sys::{Uint8Array, Array};
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Deserialize, Serialize, Clone, PartialEq)]
struct Flashcard {
    word: String,
    pinyin: Option<String>,
    translation: String,
    #[serde(default)]
    known: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum FlashcardStage {
    First,
    Second,
    Third,
}

#[derive(Clone, Copy, PartialEq)]
enum StudyDirection {
    Normal,
    Reverse,
}

#[function_component(App)]
fn app() -> Html {
    let flashcards = use_state(|| vec![]);
    let known_cards = use_state(|| vec![]);
    let current_index = use_state(|| 0usize);
    let stage = use_state(|| FlashcardStage::First);
    let direction = use_state(|| StudyDirection::Normal);
    let _reader_handle = use_state(|| None::<FileReader>);

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

            <input type="file" accept=".csv" onchange={on_file_select}/>
            <div style="margin-top: 15px;">
                <button onclick={toggle_direction.clone()}>
                    {
                        match *direction {
                            StudyDirection::Normal => "Switch to Translation → Pinyin → Character",
                            StudyDirection::Reverse => "Switch to Character → Pinyin → Translation",
                        }
                    }
                </button>
                <button onclick={update_information.clone()} style="margin-left: 10px;">
                    {"Update Information 💾"}
                </button>
                <button onclick={randomize_cards.clone()} style="margin-left: 10px;">
                    {"🔀 Randomize"}
                </button>
            </div>

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
                            }}>{"Restore"}</button>
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
