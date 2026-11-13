//! Browser-only WebAssembly frontend for Cadiotheka, built with Leptos (CSR).
//!
//! The crate registers every module explicitly; there are no `mod.rs` files.

/// Top-level application component and state.
pub mod app;

/// Presentation-layer components, grouped by kind (cards, icons, sections, ui).
pub mod components {
    /// Card-style presentations of domain entities.
    pub mod cards {
        /// Project card and its properties.
        pub mod project;
    }

    /// Inline SVG icon set.
    pub mod icons;

    /// Page-level sections that compose the layout.
    pub mod sections {
        /// Site footer.
        pub mod footer;
        /// Site header with navigation.
        pub mod header;
        /// Paginated project listing.
        pub mod projects;
    }

    /// Reusable interface widgets.
    pub mod ui {
        /// "Buy me a coffee" support link.
        pub mod buy_me_a_coffee;
        /// Decorative corner bracket frame.
        pub mod corner_frame;
        /// Transition and fade effects.
        pub mod effects {
            /// Scroll-triggered section fade overlay.
            pub mod section_fade;
        }
        /// Project logo.
        pub mod logo;
        /// Read-only Markdown rendering.
        pub mod markdown;
        /// Markdown editing widget used in the project modal.
        pub mod markdown_editor;
        /// Interactive 3D IFC viewer.
        pub mod three_d_viewer;
        /// Orientation gizmo for the 3D viewer.
        pub mod view_gizmo;
        /// Dialog-style overlays.
        pub mod modals {
            /// Modal for creating a new project.
            pub mod add_project;
            /// Shared modal shell, backdrop, and keyboard handling.
            pub mod base;
            /// Sign-in modal.
            pub mod login;
            /// Account profile modal.
            pub mod profile;
            /// Project details modal.
            pub mod project;
            /// 3D viewer configuration modal.
            pub mod viewer_settings;
        }
        /// Row that collapses overflowing children into a menu.
        pub mod overflow_row;
        /// Page number navigation.
        pub mod pagination;
        /// Transient notification toast.
        pub mod toast;
        /// Sliding on/off switch.
        pub mod toggle;
        /// Icon button used in editor toolbars.
        pub mod toolbar_button;
        /// Cloudflare Turnstile challenge widget.
        pub mod turnstile;
    }

    pub use sections::footer::Footer;
    pub use sections::header::Header;
    pub use sections::projects::ProjectsSection;

    pub use icons::Icon;
    pub use icons::Icon::{
        ICON_ARROW_DOWN, ICON_ARROW_DOWN_LEFT, ICON_ARROW_DOWN_RIGHT, ICON_ARROW_LEFT,
        ICON_ARROW_RIGHT, ICON_ARROW_UP, ICON_ARROW_UP_LEFT, ICON_ARROW_UP_RIGHT, ICON_AXES,
        ICON_BOLD, ICON_CODE, ICON_FULLSCREEN_ENTER, ICON_FULLSCREEN_EXIT, ICON_GEAR, ICON_GIZMO,
        ICON_HEADING, ICON_ITALIC, ICON_LINK, ICON_LIST_BULLET, ICON_LIST_NUMBERED, ICON_RESET,
        ICON_TASK,
    };
    pub use ui::buy_me_a_coffee::BuyMeACoffeeLogo;
    pub use ui::corner_frame::CornerFrame;
    pub use ui::effects::section_fade::FadeOverlay;
    pub use ui::logo::Logo;
    pub use ui::markdown::MarkdownView;
    pub use ui::markdown_editor::MarkdownEditor;
    pub use ui::modals::add_project::AddProjectModal;
    pub use ui::modals::base::BaseModal;
    pub use ui::modals::login::LoginModal;
    pub use ui::modals::profile::ProfileModal;
    pub use ui::modals::project::{ProjectDetailsTab, ProjectModal};
    pub use ui::modals::viewer_settings::ViewerSettingsModal;
    pub use ui::overflow_row::OverflowRow;
    pub use ui::pagination::Pagination;
    pub use ui::three_d_viewer::{IfcViewer, IfcViewerState};
    pub use ui::toast::Toast;
    pub use ui::toggle::{ToggleSlider, ToggleSliderWithSlashLabel};
    pub use ui::toolbar_button::ToolbarButton;
    pub use ui::turnstile::{TurnstileWidget, reset_turnstile, turnstile_response};
    pub use ui::view_gizmo::{GizmoPosition, ViewGizmo, ViewGizmoDirection};
}

/// Leptos reactive contexts shared across the component tree.
pub mod contexts {
    /// Accounts list fetched from the backend.
    pub mod accounts;
    /// Add-project modal visibility.
    pub mod add_project;
    /// Signed-in user state.
    pub mod current_user;
    /// Responsive layout state.
    pub mod layout;
    /// Login modal visibility.
    pub mod login;
    /// Content metadata lookup state.
    pub mod metadata;
    /// Profile modal visibility and target account.
    pub mod profile;
    /// Project details modal state.
    pub mod project_ctx;
    /// Project listing state.
    pub mod projects;
    /// Search query state.
    pub mod search;
    /// Toast notification state.
    pub mod toast;

    pub use accounts::AccountsContext;
    pub use add_project::AddProjectModalContext;
    pub use current_user::CurrentUserContext;
    pub use layout::LayoutContext;
    pub use login::LoginModalContext;
    pub use metadata::MetadataContext;
    pub use profile::ProfileModalContext;
    pub use project_ctx::ProjectModalContext;
    pub use projects::ProjectsContext;
    pub use search::SearchContext;
    pub use toast::ToastContext;
}

/// API data models and fetchers that talk to the backend.
pub mod data {
    /// Account records and their roles.
    pub mod account;
    /// Shared request error type.
    pub mod error;
    /// Project CRUD and version endpoints.
    pub mod project_api;
    /// Project domain types and payload helpers.
    pub mod project_types;

    pub use account::{AccountData, AccountRole, fetch_accounts};
    pub use error::RequestError;
    pub use project_api::*;
    pub use project_types::{
        ProjectCreationResult, ProjectData, ProjectVersion, VersionState, VersionUploadResponse,
        ifc_download_url, latest_visible_ifc_url, new_project_payload,
    };
}

/// Client-side search and suggestion logic.
pub mod engines {
    /// Card filtering.
    pub mod filter;
    /// Search query parsing and sorting.
    pub mod query;
    /// Search suggestion generation.
    pub mod suggestions;

    pub use filter::SearchEngine;
    pub use query::{ParsedQuery, SortBy, SortOrder, SortSelection, parse_query};
    pub use suggestions::{Suggestion, SuggestionKind, from_cards};
}

/// Hardcoded tag and version-state metadata.
pub mod metadata {
    /// Content tag enum, labels, and colors.
    pub mod tags;
    /// Version lifecycle states.
    pub mod version_state;

    pub use tags::{Tag, tag_color, tag_label};
    pub use version_state::VersionState;
}

/// Three.js-backed IFC model viewer internals.
pub mod three_d_viewer {
    /// Orbit camera controls.
    pub mod controls;
    /// Scene environment and lighting setup.
    pub mod environment;
    /// Per-object metadata lookup.
    pub mod primitive_metadata;
    /// Pointer picking against rendered geometry.
    pub mod raycast;
    /// Three.js renderer bootstrap.
    pub mod renderer;
    /// Scene graph construction.
    pub mod scene;
    /// Viewer view-direction and theme state.
    pub mod state;
    /// Model upload handling.
    pub mod upload;

    pub use controls::OrbitControls;
    pub use primitive_metadata::{ObjectHit, PrimitiveMetadata, fetch_metadata};
    pub use raycast::RaycastHit;
    pub use renderer::Renderer;
    pub use state::{ViewDirection, ViewState, ViewerTheme};
}

/// Small shared helpers for browser APIs, formatting, and storage.
pub mod utils {
    /// Color parsing and conversion.
    pub mod color;
    /// Non-`HttpOnly` cookie reading.
    pub mod cookie;
    /// DOM querying and event helpers.
    pub mod dom;
    /// Number, date, and size formatting.
    pub mod format;
    /// Geometric math helpers.
    pub mod math;
    /// WebAuthn passkey registration and login.
    pub mod passkey;
    /// Local and session storage wrappers.
    pub mod storage;
    /// URL parsing and construction.
    pub mod url;

    pub use color::*;
    pub use cookie::{cookie_value, has_session_hint};
    pub use dom::*;
    pub use format::*;
    pub use math::*;
    pub use passkey::{
        PasskeyInfo, delete_passkey, fetch_passkeys, login_with_passkey, passkey_supported,
        register_passkey,
    };
    pub use storage::*;
    pub use url::*;
}

pub use app::App;
