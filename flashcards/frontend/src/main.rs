use yew::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use reqwasm::http::Request;

#[derive(Deserialize, Clone, PartialEq)]
struct Flashcard {
    word: String,
    pinyin: Option<String>,
    translation: String,
}

#[derive(Clone, Copy, PartialEq)]
enum FlashcardStage {
    Word,
    Pinyin,
    Translation,
}

#[function_component(App)]
fn app() -> Html {
    let flashcards = use_state(|| vec![]);
    let current_index = use_state(|| 0usize);
    let stage = use_state(|| FlashcardStage::Word);

    // --- Load flashcards from backend ---
    let load_cards = {
        let flashcards = flashcards.clone();
        Callback::from(move |_| {
            let flashcards = flashcards.clone();
            spawn_local(async move {
                let resp = Request::get("http://127.0.0.1:8080/api/import")
                    .send()
                    .await
                    .unwrap()
                    .json::<Vec<Flashcard>>()
                    .await
                    .unwrap();
                flashcards.set(resp);
            });
        })
    };

    // --- Cycle through flashcard stages on click ---
    let on_card_click = {
        let stage = stage.clone();
        Callback::from(move |_| {
            stage.set(match *stage {
                FlashcardStage::Word => FlashcardStage::Pinyin,
                FlashcardStage::Pinyin => FlashcardStage::Translation,
                FlashcardStage::Translation => FlashcardStage::Word,
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
                stage.set(FlashcardStage::Word);
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
                stage.set(FlashcardStage::Word);
            }
        })
    };

    // --- Determine what to display ---
    let content = if !flashcards.is_empty() {
        let card = &flashcards[*current_index];
        let display_text = match *stage {
            FlashcardStage::Word => card.word.clone(),
            FlashcardStage::Pinyin => card.pinyin.clone().unwrap_or_default(),
            FlashcardStage::Translation => card.translation.clone(),
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
        html! { <p>{"Click 'Load Flashcards' to start."}</p> }
    };

    // --- Render whole app ---
    html! {
        <div style="font-family: sans-serif; text-align: center; margin-top: 40px;">
            <h1>{"Language Flashcards 🈶"}</h1>
            <button onclick={load_cards}>{"Load Flashcards"}</button>
            { content }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
