// Layout sections
pub mod sections {
    pub mod footer;
    pub mod header;
    pub mod landing;
    pub mod projects;
    pub mod teamdeck;
}

// Card components
pub mod cards {
    pub mod projectcard;
    pub mod teamcard;
}

// Visual effects and backgrounds
pub mod effects {
    pub mod blueprintgrid;
    pub mod hexgrid;
    pub mod parallax;
    pub mod sectionfade;
    pub mod wireframes;
}

// Animation components
pub mod animations {
    pub mod linedraw;
}

// UI primitives and utilities
pub mod ui {
    pub mod codeblock;
    pub mod cornerframe;
    pub mod datagrid;
}

// Re-export commonly used components for convenience
pub use sections::footer::Footer;
pub use sections::header::Header;
pub use sections::landing::LandingSection;
pub use sections::projects::ProjectsSection;
pub use sections::teamdeck::TeamDeckSection;

pub use cards::projectcard::{ProjectCard, ProjectCardProperties};
pub use cards::teamcard::{TeamCard, TeamCardProperties};

pub use effects::blueprintgrid::BlueprintGrid;
pub use effects::hexgrid::HexPattern;
pub use effects::parallax::ParallaxLanding;
pub use effects::sectionfade::FadeOverlay;
pub use effects::wireframes::WireframeShapes;

pub use animations::linedraw::{LineDraw, LineDrawHero};

pub use ui::codeblock::CodeBlock;
pub use ui::cornerframe::CornerFrame;
pub use ui::datagrid::data_grid;
