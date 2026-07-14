// Layout sections
pub mod sections {
    pub mod footer;
    pub mod header;
    pub mod projects;
}

// Card components
pub mod cards {
    pub mod projectcard;
}

// Visual effects and backgrounds
pub mod effects {
    pub mod sectionfade;
}

// UI primitives and utilities
pub mod ui {
    pub mod cornerframe;
    pub mod datagrid;
}

// Re-export commonly used components for convenience
pub use sections::footer::Footer;
pub use sections::header::Header;
pub use sections::projects::ProjectsSection;

pub use cards::projectcard::{ProjectCard, ProjectCardProperties};

pub use effects::sectionfade::FadeOverlay;

pub use ui::cornerframe::CornerFrame;
pub use ui::datagrid::data_grid;
