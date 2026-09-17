use serde::Serialize;
#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
enum OcrConfirmResult {
    Done,
    Tesseract { image_data_url: String },
}
fn main() {
    let v = OcrConfirmResult::Tesseract { image_data_url: "data:image/png;base64,xx".into() };
    println!("{}", serde_json::to_string(&v).unwrap());
}
