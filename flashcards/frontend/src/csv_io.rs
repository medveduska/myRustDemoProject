use csv::{IntoInnerError, Writer};
use js_sys::{Array, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, Url};

use crate::model::Flashcard;

pub fn parse_flashcards_from_csv(csv_data: &str) -> Vec<Flashcard> {
    let trimmed_data = csv_data.trim_start_matches('\u{FEFF}');
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(trimmed_data.as_bytes());

    reader
        .records()
        .filter_map(Result::ok)
        .map(|record| Flashcard {
            word: record.get(0).unwrap_or_default().to_string(),
            pinyin: record.get(1).map(str::to_owned),
            translation: record.get(2).unwrap_or_default().to_string(),
            known: record
                .get(3)
                .map(|value| value.trim().eq_ignore_ascii_case("true"))
                .unwrap_or(false),
        })
        .collect()
}

pub fn export_flashcards_csv<'a>(
    cards: impl Iterator<Item = &'a Flashcard>,
) -> Result<Vec<u8>, IntoInnerError<Writer<Vec<u8>>>> {
    let mut writer = csv::Writer::from_writer(Vec::new());

    for card in cards {
        let record = [
            card.word.as_str(),
            card.pinyin.as_deref().unwrap_or_default(),
            card.translation.as_str(),
            if card.known { "true" } else { "false" },
        ];
        let _ = writer.write_record(record);
    }

    let mut csv_bytes = writer.into_inner()?;
    let mut result = Vec::with_capacity(3 + csv_bytes.len());
    result.extend_from_slice(b"\xef\xbb\xbf");
    result.append(&mut csv_bytes);
    Ok(result)
}

pub fn trigger_csv_download(bytes: &[u8], file_name: &str) -> Result<(), JsValue> {
    let array = Uint8Array::from(bytes);
    let blob_parts = Array::new();
    blob_parts.push(&array.buffer());
    let blob = Blob::new_with_u8_array_sequence(&blob_parts)?;
    let url = Url::create_object_url_with_blob(&blob)?;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document unavailable"))?;
    let anchor = document.create_element("a")?;
    anchor.set_attribute("href", &url)?;
    anchor.set_attribute("download", file_name)?;

    let anchor: web_sys::HtmlElement = anchor.dyn_into()?;
    anchor.click();
    Url::revoke_object_url(&url)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{export_flashcards_csv, parse_flashcards_from_csv};

    #[test]
    fn parses_known_column_from_csv() {
        let cards = parse_flashcards_from_csv("阿姨,āyí,aunt,true\n啊,a,ah,false\n");

        assert_eq!(cards.len(), 2);
        assert!(cards[0].known);
        assert!(!cards[1].known);
        assert_eq!(cards[0].translation, "aunt");
    }

    #[test]
    fn exports_known_state_to_csv() {
        let cards = parse_flashcards_from_csv("阿姨,āyí,aunt,true\n");
        let bytes = export_flashcards_csv(cards.iter()).expect("csv export should succeed");
        let csv = String::from_utf8(bytes).expect("csv should be utf-8");

        assert!(csv.contains("阿姨,āyí,aunt,true"));
    }
}
