//! User-facing text strings for the Cadiotheka hub.

/// Strings for the header UI.
pub struct Header;

impl Header {
    pub const HEADER: &str = "Cadiotheka";
    pub const HUB_BUTTON: &str = "Hub";
    pub const HUB_ICON: &str = "🏠";
}

/// Strings for the footer UI.
pub struct Footer;

impl Footer {
    pub const COPYRIGHT_PREFIX: &str = "© 2026 ";
    pub const COPYRIGHT_OWNER: &str = "Xodium";
    pub const COPYRIGHT_URL: &str = "https://xodium.org/";
    pub const COPYRIGHT_SUFFIX: &str = ". ";
    pub const POWERED_BY_PREFIX: &str = "Powered by ";
    pub const AND: &str = " and ";
    pub const PERIOD: &str = ".";
    pub const EGUI_LABEL: &str = "egui";
    pub const EGUI_URL: &str = "https://github.com/emilk/egui";
    pub const EFRAME_LABEL: &str = "eframe";
    pub const EFRAME_URL: &str = "https://github.com/emilk/egui/tree/master/crates/eframe";
}

/// Strings for the search bar UI.
pub struct SearchBar;

impl SearchBar {
    pub const PLACEHOLDER: &str = "Navigate...";
}

/// Strings for the grid UI.
pub struct Grid;

impl Grid {
    pub const EMPTY_TITLE: &str = "No results";
    pub const EMPTY_MESSAGE: &str = "No cards match your current search.";
    pub const CLEAR_SEARCH: &str = "Clear search";
}

/// Strings for the project popup UI.
pub struct ProjectPopup;

impl ProjectPopup {
    pub const CLOSE: &str = "x";
}

/// Strings for the hub UI.
pub struct Hub;

impl Hub {
    pub const LOADING_TITLE: &str = "Loading catalog…";
    pub const LOADING_MESSAGE: &str = "Fetching the latest CAD content.";
    pub const ERROR_TITLE: &str = "Failed to load";
    pub const ERROR_MESSAGE_PREFIX: &str = "Could not load the card catalog: ";
    pub const RETRY: &str = "Retry";
}

/// Strings for startup and web errors.
pub struct WebError;

impl WebError {
    pub const NO_WINDOW: &str = "no window";
    pub const NO_DOCUMENT: &str = "no document";
    pub const CANVAS_NOT_FOUND: &str = "failed to find the_canvas_id";
    pub const CANVAS_NOT_HTML: &str = "the_canvas_id was not a HtmlCanvasElement";
    pub const CRASH_MESSAGE: &str = "The app has crashed. See the developer console for details.";
    pub const STARTUP_ERROR: &str = "failed to start eframe";
}
