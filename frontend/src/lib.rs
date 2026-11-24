//! Browser-only WebAssembly frontend for Cadiotheka, built with Leptos (CSR).
//!
//! The crate registers every module explicitly; there are no `mod.rs` files.

/// Top-level application component and state.
pub mod app;

/// Presentation-layer components, grouped by kind (layout, ui).
pub mod components {
    /// Page-level layout components.
    pub mod layout {
        /// Site footer.
        pub mod footer;
        /// Site header with navigation.
        pub mod header;
        /// Paginated project listing.
        pub mod projects;
    }

    /// Reusable interface widgets.
    pub mod ui {
        /// Decorative corner bracket frame.
        pub mod corner_frame;
        /// SVG icons, one component per file.
        pub mod icons {
            /// Arrow pointing down.
            pub mod arrow_down;
            /// Diagonal arrow pointing down-left.
            pub mod arrow_down_left;
            /// Diagonal arrow pointing down-right.
            pub mod arrow_down_right;
            /// Arrow pointing left.
            pub mod arrow_left;
            /// Arrow pointing right.
            pub mod arrow_right;
            /// Arrow pointing up.
            pub mod arrow_up;
            /// Diagonal arrow pointing up-left.
            pub mod arrow_up_left;
            /// Diagonal arrow pointing up-right.
            pub mod arrow_up_right;
            /// Compass-style axes indicator.
            pub mod axes;
            /// Bold-text mark.
            pub mod bold;
            /// Buy Me a Coffee cup.
            pub mod buy_me_a_coffee;
            /// Left-pointing chevron for pagination.
            pub mod chevron_left;
            /// Right-pointing chevron for pagination.
            pub mod chevron_right;
            /// Clock face, for relative timestamps.
            pub mod clock;
            /// X glyph to close panels and dialogs.
            pub mod close;
            /// Angle-bracket code mark.
            pub mod code;
            /// Solid cube.
            pub mod cube;
            /// Diagonal stripe over an unavailable tool.
            pub mod disabled_stripe;
            /// Download arrow with its tray.
            pub mod download_arrow;
            /// File with a plus, standing in for an IFC document.
            pub mod file_plus;
            /// Enter-fullscreen corners.
            pub mod fullscreen_enter;
            /// Exit-fullscreen corners.
            pub mod fullscreen_exit;
            /// Settings gear.
            pub mod gear;
            /// GitHub Octocat.
            pub mod github;
            /// GitHub Octocat in a 24x24 box, for buttons and list rows.
            pub mod github_mark;
            /// Compass gizmo.
            pub mod gizmo;
            /// Circled G badge for a linked Google account.
            pub mod google_badge;
            /// Google's four-colour G mark.
            pub mod google_mark;
            /// Heading mark.
            pub mod heading;
            /// Heart, filled or outlined for the favorite state.
            pub mod heart;
            /// Italic-text mark.
            pub mod italic;
            /// Chain-link mark.
            pub mod link;
            /// Bulleted list.
            pub mod list_bullet;
            /// Numbered list.
            pub mod list_numbered;
            /// Mail envelope beside an account email.
            pub mod mail;
            /// Minus sign for remove and disconnect overlays.
            pub mod minus;
            /// Phosphor key glyph for passkey actions.
            pub mod passkey;
            /// Pencil for edit affordances.
            pub mod pencil;
            /// Plus sign for add/create actions.
            pub mod plus;
            /// Reset-to-defaults arrow loop.
            pub mod reset;
            /// Magnifier for the search button.
            pub mod search;
            /// Log-in arrow with its door.
            pub mod sign_in;
            /// Checklist mark.
            pub mod task;
            /// Trash can for delete actions.
            pub mod trash;
            /// Warning triangle for the danger zone.
            pub mod warning;
            /// Wireframe cube.
            pub mod wireframe;

            pub use arrow_down::ArrowDownIcon;
            pub use arrow_down_left::ArrowDownLeftIcon;
            pub use arrow_down_right::ArrowDownRightIcon;
            pub use arrow_left::ArrowLeftIcon;
            pub use arrow_right::ArrowRightIcon;
            pub use arrow_up::ArrowUpIcon;
            pub use arrow_up_left::ArrowUpLeftIcon;
            pub use arrow_up_right::ArrowUpRightIcon;
            pub use axes::AxesIcon;
            pub use bold::BoldIcon;
            pub use buy_me_a_coffee::BuyMeACoffeeIcon;
            pub use chevron_left::ChevronLeftIcon;
            pub use chevron_right::ChevronRightIcon;
            pub use clock::ClockIcon;
            pub use close::CloseIcon;
            pub use code::CodeIcon;
            pub use cube::CubeIcon;
            pub use disabled_stripe::DisabledStripeIcon;
            pub use download_arrow::DownloadArrowIcon;
            pub use file_plus::FilePlusIcon;
            pub use fullscreen_enter::FullscreenEnterIcon;
            pub use fullscreen_exit::FullscreenExitIcon;
            pub use gear::GearIcon;
            pub use github::GitHubIcon;
            pub use github_mark::GitHubMarkIcon;
            pub use gizmo::GizmoIcon;
            pub use google_badge::GoogleBadgeIcon;
            pub use google_mark::GoogleMarkIcon;
            pub use heading::HeadingIcon;
            pub use heart::HeartIcon;
            pub use italic::ItalicIcon;
            pub use link::LinkIcon;
            pub use list_bullet::ListBulletIcon;
            pub use list_numbered::ListNumberedIcon;
            pub use mail::MailIcon;
            pub use minus::MinusIcon;
            pub use passkey::PasskeyIcon;
            pub use pencil::PencilIcon;
            pub use plus::PlusIcon;
            pub use reset::ResetIcon;
            pub use search::SearchIcon;
            pub use sign_in::SignInIcon;
            pub use task::TaskIcon;
            pub use trash::TrashIcon;
            pub use warning::WarningIcon;
            pub use wireframe::WireframeIcon;
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
        /// Project card and the properties it renders.
        pub mod project_card;
        /// Transient notification toast.
        pub mod toast;
        /// Sliding on/off switch.
        pub mod toggle;
        /// Icon button used in editor toolbars.
        pub mod toolbar_button;
    }

    pub use layout::footer::Footer;
    pub use layout::header::Header;
    pub use layout::projects::ProjectsSection;

    pub use ui::corner_frame::CornerFrame;
    pub use ui::icons::{buy_me_a_coffee::BuyMeACoffeeIcon, github::GitHubIcon};
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
    pub use ui::project_card::{ProjectCard, ProjectCardProperties};
    pub use ui::three_d_viewer::{IfcViewer, IfcViewerState};
    pub use ui::toast::Toast;
    pub use ui::toggle::{ToggleSlider, ToggleSliderWithSlashLabel};
    pub use ui::toolbar_button::ToolbarButton;
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
        Project, ProjectCreationResult, ProjectVersion, VersionState, VersionUploadResponse,
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
