pub mod app;

pub mod components {
    pub mod cards {
        pub mod project;
    }

    pub mod icons;

    pub mod sections {
        pub mod footer;
        pub mod header;
        pub mod projects;
    }

    pub mod ui {
        pub mod buy_me_a_coffee;
        pub mod corner_frame;
        pub mod effects {
            pub mod section_fade;
        }
        pub mod logo;
        pub mod markdown;
        pub mod markdown_editor;
        pub mod three_d_viewer;
        pub mod view_gizmo;
        pub mod modals {
            pub mod add_project;
            pub mod base;
            pub mod login;
            pub mod profile;
            pub mod project;
        }
        pub mod overflow_row;
        pub mod pagination;
        pub mod toast;
        pub mod toggle;
        pub mod toolbar_button;
        pub mod turnstile;
    }

    pub use sections::footer::Footer;
    pub use sections::header::Header;
    pub use sections::projects::ProjectsSection;

    pub use icons::Icon;
    pub use icons::Icon::{
        ICON_AXES, ICON_BOLD, ICON_CODE, ICON_FULLSCREEN_ENTER, ICON_FULLSCREEN_EXIT, ICON_GIZMO,
        ICON_GRID, ICON_HEADING, ICON_ITALIC, ICON_LINK, ICON_LIST_BULLET, ICON_LIST_NUMBERED,
        ICON_RESET, ICON_TASK,
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
    pub use ui::overflow_row::OverflowRow;
    pub use ui::pagination::Pagination;
    pub use ui::three_d_viewer::{IfcViewer, IfcViewerState};
    pub use ui::toast::Toast;
    pub use ui::toggle::{ToggleSlider, ToggleSliderWithSlashLabel};
    pub use ui::toolbar_button::ToolbarButton;
    pub use ui::turnstile::{TurnstileWidget, reset_turnstile, turnstile_response};
    pub use ui::view_gizmo::{ViewGizmo, ViewGizmoDirection};
}

pub mod contexts {
    pub mod accounts;
    pub mod add_project;
    pub mod current_user;
    pub mod layout;
    pub mod login;
    pub mod metadata;
    pub mod profile;
    pub mod project_ctx;
    pub mod projects;
    pub mod search;
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

pub mod data {
    pub mod account;
    pub mod error;
    pub mod project_api;
    pub mod project_types;

    pub use account::{AccountData, AccountRole, fetch_accounts};
    pub use error::RequestError;
    pub use project_api::*;
    pub use project_types::{
        ProjectCreationResult, ProjectData, ProjectVersion, VersionState, VersionUploadResponse,
        ifc_download_url, latest_visible_ifc_url, new_project_payload,
    };
}

pub mod engines {
    pub mod filter;
    pub mod query;
    pub mod suggestions;

    pub use filter::SearchEngine;
    pub use query::{ParsedQuery, SortBy, SortOrder, SortSelection, parse_query};
    pub use suggestions::{Suggestion, SuggestionKind, from_cards};
}

pub mod metadata {
    pub mod tags;
    pub mod version_state;

    pub use tags::{Tag, tag_color, tag_label};
    pub use version_state::VersionState;
}

pub mod three_d_viewer {
    pub mod controls;
    pub mod environment;
    pub mod renderer;
    pub mod scene;
    pub mod state;
    pub mod upload;

    pub use controls::OrbitControls;
    pub use renderer::Renderer;
    pub use state::{ViewDirection, ViewState, ViewerSettings, ViewerTheme};
}

pub use three_d_viewer::{
    OrbitControls, Renderer, ViewDirection, ViewState, ViewerSettings, ViewerTheme,
};

pub mod utils {
    pub mod color;
    pub mod dom;
    pub mod format;
    pub mod math;
    pub mod storage;
    pub mod url;

    pub use color::*;
    pub use dom::*;
    pub use format::*;
    pub use math::*;
    pub use storage::*;
    pub use url::*;
}

pub use app::App;
