#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod app;

pub mod components {
    pub mod cards {
        pub mod project;
    }

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
        pub mod modals {
            pub mod add_project;
            pub mod login;
            pub mod profile;
            pub mod project;
            pub mod search;
        }
        pub mod overflow_row;
        pub mod project_icon_picker;
        pub mod toast;
        pub mod toggle;
        pub mod toolbar_button;
        pub mod turnstile;
    }

    pub use sections::footer::Footer;
    pub use sections::header::Header;
    pub use sections::projects::ProjectsSection;

    pub use ui::buy_me_a_coffee::BuyMeACoffeeLogo;
    pub use ui::corner_frame::CornerFrame;
    pub use ui::effects::section_fade::FadeOverlay;
    pub use ui::logo::Logo;
    pub use ui::markdown::MarkdownView;
    pub use ui::markdown_editor::MarkdownEditor;
    pub use ui::modals::add_project::AddProjectModal;
    pub use ui::modals::login::LoginModal;
    pub use ui::modals::profile::ProfileModal;
    pub use ui::modals::project::{ProjectDetailsTab, ProjectModal};
    pub use ui::modals::search::SearchModal;
    pub use ui::overflow_row::OverflowRow;
    pub use ui::project_icon_picker::ProjectIconPicker;
    pub use ui::three_d_viewer::{IfcViewer, IfcViewerState};
    pub use ui::toast::Toast;
    pub use ui::toggle::{ToggleSlider, ToggleSliderWithSlashLabel};
    pub use ui::toolbar_button::ToolbarButton;
    pub use ui::turnstile::{TurnstileWidget, reset_turnstile, turnstile_response};
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
}

pub mod data {
    pub mod account;
    pub mod error;
    pub mod metadata;
    pub mod project;
    pub mod project_api;
    pub mod project_types;

    pub use account::{AccountData, AccountRole, fetch_accounts};
    pub use error::RequestError;
    pub use metadata::{fetch_platforms, fetch_tags};
    pub use project::{
        IconUrl, ProjectCreationResult, ProjectData, ProjectPatch, convert_project_glb,
        create_project, delete_project, delete_project_ifc, fetch_projects,
        increment_project_downloads, new_project_payload, toggle_project_favorite, update_project,
        update_project_collaborators, update_project_description, update_project_platforms,
        update_project_tags, update_project_title, upload_project_icon, upload_project_ifc,
    };
}

pub mod engines;

pub mod metadata {
    pub mod platforms;
    pub mod tags;

    pub use platforms::{platform_color, platform_label};
    pub use tags::{tag_color, tag_label};
}

pub mod three_d_viewer {
    pub mod controls;
    pub mod renderer;
    pub mod scene;
    pub mod state;
    pub mod upload;

    pub use controls::OrbitControls;
    pub use renderer::Renderer;
    pub use state::{ViewState, ViewerSettings, ViewerTheme};
}

pub use three_d_viewer::{OrbitControls, Renderer, ViewState, ViewerSettings, ViewerTheme};

pub mod utils {
    pub mod color;
    pub mod dom;
    pub mod format;
    pub mod math;
    pub mod url;

    pub use color::*;
    pub use dom::*;
    pub use format::*;
    pub use math::*;
    pub use url::*;
}

pub use app::App;
