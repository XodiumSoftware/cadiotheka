//! User-facing text strings for the Cadiotheka hub.

/// Strings for the hub UI.
pub struct Hub;

impl Hub {
    /// Header text shown at the top of the hub UI.
    pub const HEADER: &str = "Cadiotheka";

    /// Label for the Hub navigation button.
    pub const HUB_BUTTON: &str = "Hub";

    /// Hub icon shown on the header navigation button.
    pub const HUB_ICON: &str = "🏠";
}

/// Strings for startup and web errors.
pub struct WebError;

impl WebError {
    /// Error message when the browser window is not available.
    pub const NO_WINDOW: &str = "no window";

    /// Error message when the browser document is not available.
    pub const NO_DOCUMENT: &str = "no document";

    /// Error message when the canvas element cannot be found.
    pub const CANVAS_NOT_FOUND: &str = "failed to find the_canvas_id";

    /// Error message when the found element is not a canvas.
    pub const CANVAS_NOT_HTML: &str = "the_canvas_id was not a HtmlCanvasElement";

    /// Message shown to the user when the app crashes on startup.
    pub const CRASH_MESSAGE: &str = "The app has crashed. See the developer console for details.";

    /// Prefix for the panic message when eframe fails to start.
    pub const STARTUP_ERROR: &str = "failed to start eframe";
}
