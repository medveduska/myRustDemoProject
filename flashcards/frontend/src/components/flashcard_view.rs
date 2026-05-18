use web_sys::{InputEvent, KeyboardEvent, MouseEvent};
use yew::prelude::*;

use crate::model::StudyMode;

#[derive(Properties, PartialEq)]
pub struct FlashcardViewProps {
    pub mode: StudyMode,
    pub card_text: Option<String>,
    pub card_state_class: Option<&'static str>,
    pub on_card_click: Callback<MouseEvent>,
    pub exercise_answer: String,
    pub on_exercise_input: Callback<InputEvent>,
    pub on_exercise_keydown: Callback<KeyboardEvent>,
    pub on_check_answer: Callback<()>,
    pub exercise_feedback: Option<String>,
    pub exercise_feedback_class: Option<&'static str>,
    pub exercise_pinyin: Option<String>,
    pub on_prev: Callback<MouseEvent>,
    pub on_mark_known: Callback<MouseEvent>,
    pub on_delete: Callback<MouseEvent>,
    pub on_next: Callback<MouseEvent>,
}

#[function_component(FlashcardView)]
pub fn flashcard_view(props: &FlashcardViewProps) -> Html {
    let Some(card_text) = props.card_text.clone() else {
        return html! { <p class="empty-note">{"No unknown flashcards remaining."}</p> };
    };

    let card_classes = classes!("flashcard", props.card_state_class);
    let feedback_classes = classes!("exercise-feedback", props.exercise_feedback_class);

    let exercise_section = if props.mode == StudyMode::Exercise {
        html! {
            <div class="exercise-controls">
                <input
                    class="text-input exercise-answer-input"
                    placeholder="Type your answer"
                    value={props.exercise_answer.clone()}
                    oninput={props.on_exercise_input.clone()}
                    onkeydown={props.on_exercise_keydown.clone()}
                />
                <button class="btn btn-primary" onclick={props.on_check_answer.reform(|_: MouseEvent| ())}>
                    {"Check"}
                </button>
                if let Some(feedback) = props.exercise_feedback.clone() {
                    <p class={feedback_classes}>{feedback}</p>
                }
                if let Some(pinyin) = props.exercise_pinyin.clone() {
                    <p class="exercise-pinyin">{format!("Pinyin: {}", pinyin)}</p>
                }
            </div>
        }
    } else {
        html! {}
    };

    html! {
        <>
            if props.mode == StudyMode::Reveal {
                <div
                    onclick={props.on_card_click.clone()}
                    class={card_classes.clone()}
                >
                    { card_text.clone() }
                </div>
            } else {
                <div class={card_classes}>
                    { card_text }
                </div>
            }

            {exercise_section}

            <div class="flashcard-actions">
                <button class="btn btn-secondary" onclick={props.on_prev.clone()}>{"<- Prev"}</button>
                <button class="btn btn-primary" onclick={props.on_mark_known.clone()}>{"Mark as Known"}</button>
                <button class="btn btn-danger" onclick={props.on_delete.clone()}>{"Delete"}</button>
                <button class="btn btn-secondary" onclick={props.on_next.clone()}>{"Next ->"}</button>
            </div>
        </>
    }
}
