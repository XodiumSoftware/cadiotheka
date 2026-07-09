//! User-facing text strings for the Cadiotheka hub.

/// Main heading shown in the hub UI.
pub const HEADING: &str = "Welcome to Cadiotheka";

/// Label for the name input field.
pub const NAME_LABEL: &str = "Your name:";

/// Label for the counter controls.
pub const COUNTER_LABEL: &str = "Counter:";

/// Button text for decrementing the counter.
pub const DECREMENT_BUTTON: &str = "-";

/// Button text for incrementing the counter.
pub const INCREMENT_BUTTON: &str = "+";

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
