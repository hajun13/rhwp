//! 줄 첫 글자부터 시작하는 본문 선택 사각형의 x 는 같은 위치의 캐럿 x 와 같아야 한다.
//!
//! `getSelectionRects` 는 줄 첫 글자부터의 선택을 단 왼쪽 끝(area_left)에서 칠해, 번호 문단은
//! 번호 폭만큼, 가운데 정렬 문단은 정렬로 밀린 거리만큼 글자보다 왼쪽을 덮었다.
//! 캐럿(`getCursorRect`)은 글자 run 의 x 라 정렬·번호·여백을 이미 포함한다.

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|e| panic!("JSON {text}: {e}"))
}

/// 문단 1(번호) · 2(가운데) · 3(보통)에 같은 글 `한한첫`을 넣는다. 문단 0 은 건드리지 않는다.
fn three_paragraphs() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("빈 문서");
    let id = doc.ensure_default_numbering();
    doc.insert_text_native(0, 0, 0, "머리").expect("문단 0 글");
    for para in 0..3 {
        let at = if para == 0 { 2 } else { 3 };
        doc.split_paragraph_native(0, para, at, None)
            .expect("문단 나눔");
        doc.insert_text_native(0, para + 1, 0, "한한첫")
            .expect("문단 글");
    }
    let numbered = format!(r#"{{"headType":"Number","paraLevel":0,"numberingId":{id}}}"#);
    doc.apply_para_format_native(0, 1, &numbered)
        .expect("번호 문단");
    doc.apply_para_format_native(0, 2, r#"{"alignment":"center"}"#)
        .expect("가운데 정렬");
    doc.apply_para_format_native(0, 3, r#"{"alignment":"left"}"#)
        .expect("왼쪽 정렬");
    doc
}

/// (캐럿 x, 첫 선택 사각형 x). 페이지가 같은지도 확인한다.
fn caret_and_first_rect_x(doc: &HwpDocument, para: u32) -> (f64, f64) {
    let caret = json(&doc.get_cursor_rect(0, para, 0).expect("캐럿"));
    let rects = json(
        &doc.get_selection_rects(0, para, 0, para, 3)
            .expect("선택 사각형"),
    );
    let first = rects
        .as_array()
        .and_then(|a| a.first())
        .expect("사각형 없음");
    assert_eq!(first["pageIndex"], caret["pageIndex"], "문단 {para}: 쪽");
    (caret["x"].as_f64().unwrap(), first["x"].as_f64().unwrap())
}

fn assert_close(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 0.5,
        "{label}: 선택 {actual:.1}, 캐럿 {expected:.1}"
    );
}

#[test]
fn line_start_selection_x_matches_caret_for_numbered_and_centered() {
    let doc = three_paragraphs();
    let (plain_caret, plain_rect) = caret_and_first_rect_x(&doc, 3);
    let (num_caret, num_rect) = caret_and_first_rect_x(&doc, 1);
    let (center_caret, center_rect) = caret_and_first_rect_x(&doc, 2);

    // 보통 문단은 예전에도 같았다. 번호·가운데 문단 캐럿은 그보다 오른쪽이어야 시험이 뜻을 가진다.
    assert_close(plain_rect, plain_caret, "보통");
    assert!(
        num_caret > plain_caret + 1.0,
        "번호 캐럿이 본문 시작 뒤여야 함"
    );
    assert!(
        center_caret > plain_caret + 1.0,
        "가운데 캐럿이 정렬로 밀려야 함"
    );

    assert_close(num_rect, num_caret, "번호");
    assert_close(center_rect, center_caret, "가운데");
}
