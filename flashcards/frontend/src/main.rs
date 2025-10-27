use yew::prelude::*;
use serde::Deserialize;
use gloo_file::callbacks::FileReader;
use gloo_file::File;
use web_sys::HtmlInputElement;

#[derive(Deserialize, Clone, PartialEq)]
struct Flashcard {
    word: String,
    pinyin: Option<String>,
    translation: String,
}

#[derive(Clone, Copy, PartialEq)]
enum FlashcardStage {
    First,
    Second,
    Third,
}

#[derive(Clone, Copy, PartialEq)]
enum StudyDirection {
    Normal,   // Word -> Pinyin -> Translation
    Reverse,  // Translation -> Pinyin -> Word
}

#[function_component(App)]
fn app() -> Html {
    let flashcards = use_state(|| vec![]);
    let current_index = use_state(|| 0usize);
    let stage = use_state(|| FlashcardStage::First);
    let direction = use_state(|| StudyDirection::Normal);
    let _reader_handle = use_state(|| None::<FileReader>);

    // --- File selection handler ---
    let on_file_select = {
        let flashcards = flashcards.clone();
        let reader_handle = _reader_handle.clone();

        Callback::from(move |event: Event| {
            let input: HtmlInputElement = event
                .target_dyn_into()
                .expect("Failed to cast file input");
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let file = File::from(file);
                    let flashcards = flashcards.clone();
                    let reader_handle = reader_handle.clone();

                    let task = gloo_file::callbacks::read_as_text(&file, move |res| {
                        if let Ok(csv_data) = res {
                            let mut rdr = csv::ReaderBuilder::new()
                                .has_headers(false)
                                .from_reader(csv_data.as_bytes());
                            let mut cards = Vec::new();
                            for record in rdr.records() {
                                if let Ok(r) = record {
                                    let word = r.get(0).unwrap_or("").to_string();
                                    let pinyin = r.get(1).map(|s| s.to_string());
                                    let translation = r.get(2).unwrap_or("").to_string();
                                    cards.push(Flashcard { word, pinyin, translation });
                                }
                            }
                            flashcards.set(cards);
                        }
                    });

                    reader_handle.set(Some(task));
                }
            }
        })
    };

    // --- Cycle within one card ---
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

    // --- Navigation buttons ---
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

    // --- Direction toggle button ---
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

    // --- Determine displayed text ---
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

                <div style="margin-top: 20px;">
                    <button onclick={prev_card.clone()}>{"← Previous"}</button>
                    <span style="margin: 0 15px;">
                        { format!("{}/{}", *current_index + 1, flashcards.len()) }
                    </span>
                    <button onclick={next_card.clone()}>{"Next →"}</button>
                </div>
            </>
        }
    } else {
        html! { <p>{"Select a CSV file to start learning."}</p> }
    };

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
            </div>
            { content }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
