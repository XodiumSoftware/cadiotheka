//! Entry point for the Cadiotheka web hub.

use cadiotheka::i18n;
use eframe::wasm_bindgen::JsCast;

/// Starts the Cadiotheka hub on the web canvas.
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async move {
        let document = web_sys::window()
            .expect(i18n::NO_WINDOW)
            .document()
            .expect(i18n::NO_DOCUMENT);

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect(i18n::CANVAS_NOT_FOUND)
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect(i18n::CANVAS_NOT_HTML);

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(cadiotheka::CadiothekaApp::default()))),
            )
            .await;

        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => loading_text.remove(),
                Err(e) => {
                    loading_text.set_inner_html(i18n::CRASH_MESSAGE);
                    panic!("{}: {e:?}", i18n::STARTUP_ERROR);
                }
            }
        }
    });
}
