use crate::components::Icon;
use crate::components::IfcViewer;
use crate::components::Pagination;
use crate::components::ViewerSettingsModal;
use crate::components::cards::project::{DownloadIcon, HeartIcon, ProjectCardProperties};
use crate::components::ui::markdown::MarkdownView;
use crate::components::ui::markdown_editor::MarkdownEditor;
use crate::components::ui::modals::base::BaseModal;
use crate::components::ui::toolbar_button::{ToolbarButton, TooltipPosition};
use crate::components::ui::view_gizmo::GizmoPosition;
use crate::three_d_viewer::ObjectHit;

use crate::contexts::{
    AccountsContext, CurrentUserContext, MetadataContext, ProfileModalContext, ProjectModalContext,
    ProjectsContext, SearchContext, ToastContext,
};
use crate::data::{
    AccountData, AccountRole, ProjectVersion, convert_project_glb, delete_project,
    delete_project_version, fetch_project_versions, fetch_projects, ifc_download_url,
    increment_project_downloads, latest_visible_ifc_url, update_project_collaborators,
    update_project_description, update_project_tags, update_project_title,
    update_project_version_state, upload_project_ifc,
};
use crate::metadata::VersionState;
use crate::metadata::tags::Tag;
use crate::utils::{
    format_number, format_version_timestamp, hex_to_srgba, placeholder_color, placeholder_letter,
    project_glb_metadata_url, project_glb_url, srgba_to_hex,
};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use shared::validation::{MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH};
use std::rc::Rc;
use three_d_asset::Srgba;

const VERSIONS_PER_PAGE: usize = 20;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProjectDetailsTab {
    Viewer3d,
    Versions,
}

const GIZMO_POSITION_KEY: &str = "gizmo_position";
const GIZMO_VISIBLE_KEY: &str = "gizmo_visible";
const AXES_VISIBLE_KEY: &str = "axes_visible";
const OBJECT_HIGHLIGHT_COLOR_KEY: &str = "object_highlight_color";
const SKYBOX_COLOR_KEY: &str = "skybox_color";

/// Default outline color used when no viewer preferences are saved.
const DEFAULT_OBJECT_HIGHLIGHT_COLOR: Srgba = Srgba::new(255, 200, 0, 255);

/// Loads the saved gizmo visibility from account viewer preferences.
fn load_gizmo_visible_from_preferences(account: Option<&AccountData>) -> bool {
    let Some(account) = account else {
        return true;
    };
    let prefs: serde_json::Value =
        serde_json::from_str(&account.viewer_preferences).unwrap_or(serde_json::json!({}));
    prefs
        .get(GIZMO_VISIBLE_KEY)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true)
}

/// Returns the account's existing viewer preferences with the gizmo visibility
/// updated, preserving any other keys that may exist in the JSON blob.
fn preferences_with_gizmo_visible(account: Option<&AccountData>, visible: bool) -> Option<String> {
    let account = account?;
    let mut prefs: serde_json::Value =
        serde_json::from_str(&account.viewer_preferences).unwrap_or(serde_json::json!({}));
    prefs[GIZMO_VISIBLE_KEY] = serde_json::json!(visible);
    serde_json::to_string(&prefs).ok()
}

/// Loads the saved axes gizmo visibility from account viewer preferences.
fn load_axes_visible_from_preferences(account: Option<&AccountData>) -> bool {
    let Some(account) = account else {
        return true;
    };
    let prefs: serde_json::Value =
        serde_json::from_str(&account.viewer_preferences).unwrap_or(serde_json::json!({}));
    prefs
        .get(AXES_VISIBLE_KEY)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true)
}

/// Returns the account's existing viewer preferences with the axes visibility
/// updated, preserving any other keys that may exist in the JSON blob.
fn preferences_with_axes_visible(account: Option<&AccountData>, visible: bool) -> Option<String> {
    let account = account?;
    let mut prefs: serde_json::Value =
        serde_json::from_str(&account.viewer_preferences).unwrap_or(serde_json::json!({}));
    prefs[AXES_VISIBLE_KEY] = serde_json::json!(visible);
    serde_json::to_string(&prefs).ok()
}

/// Loads the saved gizmo position from account viewer preferences.
fn load_gizmo_position_from_preferences(account: Option<&AccountData>) -> GizmoPosition {
    let Some(account) = account else {
        return GizmoPosition::TopRight;
    };
    let prefs: serde_json::Value =
        serde_json::from_str(&account.viewer_preferences).unwrap_or(serde_json::json!({}));
    prefs
        .get(GIZMO_POSITION_KEY)
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "top_left" => Some(GizmoPosition::TopLeft),
            "top_right" => Some(GizmoPosition::TopRight),
            "bottom_left" => Some(GizmoPosition::BottomLeft),
            "bottom_right" => Some(GizmoPosition::BottomRight),
            "top_center" => Some(GizmoPosition::TopCenter),
            "right_center" => Some(GizmoPosition::RightCenter),
            "bottom_center" => Some(GizmoPosition::BottomCenter),
            "left_center" => Some(GizmoPosition::LeftCenter),
            _ => None,
        })
        .unwrap_or(GizmoPosition::TopRight)
}

/// Wire id used to persist a [`GizmoPosition`] in the viewer preferences JSON.
fn gizmo_position_wire_id(position: GizmoPosition) -> &'static str {
    match position {
        GizmoPosition::TopLeft => "top_left",
        GizmoPosition::TopRight => "top_right",
        GizmoPosition::BottomLeft => "bottom_left",
        GizmoPosition::BottomRight => "bottom_right",
        GizmoPosition::TopCenter => "top_center",
        GizmoPosition::RightCenter => "right_center",
        GizmoPosition::BottomCenter => "bottom_center",
        GizmoPosition::LeftCenter => "left_center",
    }
}

/// Returns the account's existing viewer preferences with the gizmo position
/// updated, preserving any other keys that may exist in the JSON blob.
fn preferences_with_gizmo_position(
    account: Option<&AccountData>,
    position: GizmoPosition,
) -> Option<String> {
    let account = account?;
    let mut prefs: serde_json::Value =
        serde_json::from_str(&account.viewer_preferences).unwrap_or(serde_json::json!({}));
    prefs[GIZMO_POSITION_KEY] = serde_json::json!(gizmo_position_wire_id(position));
    serde_json::to_string(&prefs).ok()
}

/// Loads the saved object highlight color from account viewer preferences.
fn load_object_highlight_color_from_preferences(account: Option<&AccountData>) -> Srgba {
    let Some(account) = account else {
        return DEFAULT_OBJECT_HIGHLIGHT_COLOR;
    };
    let prefs: serde_json::Value =
        serde_json::from_str(&account.viewer_preferences).unwrap_or(serde_json::json!({}));
    prefs
        .get(OBJECT_HIGHLIGHT_COLOR_KEY)
        .and_then(serde_json::Value::as_str)
        .and_then(hex_to_srgba)
        .unwrap_or(DEFAULT_OBJECT_HIGHLIGHT_COLOR)
}

/// Returns the account's existing viewer preferences with the object highlight color
/// updated, preserving any other keys that may exist in the JSON blob.
fn preferences_with_object_highlight_color(
    account: Option<&AccountData>,
    color: Srgba,
) -> Option<String> {
    let account = account?;
    let mut prefs: serde_json::Value =
        serde_json::from_str(&account.viewer_preferences).unwrap_or(serde_json::json!({}));
    prefs[OBJECT_HIGHLIGHT_COLOR_KEY] = serde_json::json!(srgba_to_hex(color));
    serde_json::to_string(&prefs).ok()
}

/// Loads the saved skybox background color from account viewer preferences.
fn load_skybox_color_from_preferences(account: Option<&AccountData>) -> Srgba {
    let Some(account) = account else {
        return Srgba::WHITE;
    };
    let prefs: serde_json::Value =
        serde_json::from_str(&account.viewer_preferences).unwrap_or(serde_json::json!({}));
    prefs
        .get(SKYBOX_COLOR_KEY)
        .and_then(serde_json::Value::as_str)
        .and_then(hex_to_srgba)
        .unwrap_or(Srgba::WHITE)
}

/// Returns the account's existing viewer preferences with the skybox background
/// color updated, preserving any other keys that may exist in the JSON blob.
fn preferences_with_skybox_color(account: Option<&AccountData>, color: Srgba) -> Option<String> {
    let account = account?;
    let mut prefs: serde_json::Value =
        serde_json::from_str(&account.viewer_preferences).unwrap_or(serde_json::json!({}));
    prefs[SKYBOX_COLOR_KEY] = serde_json::json!(srgba_to_hex(color));
    serde_json::to_string(&prefs).ok()
}

/// Pipeline status for converting a project's IFC model to a viewable GLB.
#[derive(Clone, Copy, PartialEq, Eq)]
enum GlbConversionStatus {
    /// No IFC model, or conversion not yet triggered.
    Idle,
    /// The IFC model is being converted to GLB on the backend.
    Converting,
    /// The GLB is ready to view.
    Ready,
    /// The IFC model produced no renderable geometry.
    NoGeometry,
    /// The conversion request failed.
    Failed,
}

/// Modal dialog that displays detailed information about a selected project.
#[component]
pub fn ProjectModal() -> impl IntoView {
    let modal = ProjectModalContext::use_context();
    let (viewer_fullscreen, set_viewer_fullscreen) = signal(false);
    let on_close = Callback::new(move |()| {
        if is_fullscreen_active() {
            if let Some(document) = leptos::web_sys::window().and_then(|w| w.document()) {
                document.exit_fullscreen();
            }
            return;
        }
        modal.close();
    });

    // Keep a stable card for the lifetime of the open modal so that updates to
    // the context card (e.g. added/removed versions) do not remount the whole
    // modal content and reset the active tab.
    let stable_card = RwSignal::new(None::<ProjectCardProperties>);
    Effect::new(move |_| {
        let maybe_card = modal.card.get();
        stable_card.update(|current| {
            let should_update = match (current.as_ref(), maybe_card.as_ref()) {
                (None, Some(_)) => true,
                (Some(current), Some(new)) if current.id != new.id => true,
                _ => false,
            };
            if should_update {
                *current = maybe_card;
            }
        });
    });

    Effect::new(move |_| {
        if !modal.open.get() {
            set_viewer_fullscreen.set(false);
            if let Some(document) = leptos::web_sys::window().and_then(|w| w.document())
                && document.fullscreen_element().is_some()
            {
                document.exit_fullscreen();
            }
        }
    });

    view! {
        <BaseModal
            open=modal.open
            on_close=on_close
            container_class=Signal::derive({
                let active_tab = modal.active_tab;
                move || {
                    if viewer_fullscreen.get() {
                        "h-[90vh] w-[90vw] max-h-[90vh] max-w-[90vw] flex flex-col".to_string()
                    } else if active_tab.get() == ProjectDetailsTab::Viewer3d {
                        "w-[95vw] max-w-[95vw] h-full max-h-[90vh] flex flex-col".to_string()
                    } else {
                        "w-full max-w-6xl h-full max-h-[90vh] flex flex-col".to_string()
                    }
                }
            })
        >
            {move || {
                let maybe_card = stable_card.get();
                match maybe_card {
                    Some(card) => view! {
                        <ProjectModalContent
                            card=card
                            viewer_fullscreen=viewer_fullscreen
                            set_viewer_fullscreen=set_viewer_fullscreen
                        />
                    }
                        .into_any(),
                    None => view! {
                        <p class="text-base-content/50 text-sm">No project selected.</p>
                    }
                        .into_any(),
                }
            }}
        </BaseModal>
    }
}

fn is_fullscreen_active() -> bool {
    leptos::web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.fullscreen_element())
        .is_some()
}

fn avatar_button(account: &AccountData, class: Option<String>) -> impl IntoView + use<> {
    let display_name = account.display_name.clone();
    let avatar_alt = format!("{display_name}'s avatar");
    let avatar_letter = placeholder_letter(&display_name);
    let avatar_bg = placeholder_color(&display_name);
    let size_class = class.unwrap_or_else(|| "w-12 h-12".to_string());
    let url = account.avatar_url.clone();
    view! {
        <div
            class=format!("{} border border-base-content/10 overflow-hidden flex items-center justify-center text-white font-bold text-lg tooltip tooltip-top {}", size_class, avatar_bg)
            data-tip=display_name.clone()
            aria-label=avatar_alt.clone()
        >
            {url.map_or_else(move || {
                view! {
                    <span>{avatar_letter.clone()}</span>
                }
                    .into_any()
            }, |url| {
                view! {
                    <img class="w-full h-full object-cover" src=url alt=avatar_alt.clone() loading="lazy" />
                }
                    .into_any()
            })}
        </div>
    }
}

fn trash_icon(class: &'static str) -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class=class
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="M3 6h18" />
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
        </svg>
    }
}

fn warning_icon(class: &'static str) -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class=class
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
            <line x1="12" y1="9" x2="12" y2="13" />
            <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
    }
}

fn edit_pencil_icon(class: &'static str) -> impl IntoView {
    view! {
        <svg
            class=class
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
        </svg>
    }
}

fn ifc_file_icon(class: &'static str, color_class: &'static str) -> impl IntoView {
    view! {
        <svg class=format!("{class} {color_class}") viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="12" y1="18" x2="12" y2="12" />
            <line x1="9" y1="15" x2="15" y2="15" />
        </svg>
    }
}

/// Modal that lets editors change a version's maturity state.
#[component]
fn VersionStateSelector(
    #[prop(into)] state: Signal<VersionState>,
    #[prop(into)] on_change: Callback<VersionState>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let on_close = Callback::new(move |()| open.set(false));

    view! {
        <>
            <button
                type="button"
                class=move || {
                    let s = state.get();
                    format!(
                        "group relative w-10 h-10 rounded-none flex items-center justify-center cursor-pointer bg-base-200/50 border {} hover:border-primary transition-colors",
                        s.border_class()
                    )
                }
                aria-label=move || format!("Current state: {}. Open state selector.", state.get().label())
                on:click=move |_| open.set(true)
            >
                {move || {
                    let s = state.get();
                    if s == VersionState::Undefined {
                        ifc_file_icon("w-5 h-5", s.color_class()).into_any()
                    } else {
                        view! {
                            <span class=format!("text-sm font-semibold {}", s.color_class())>{s.letter()}</span>
                        }
                            .into_any()
                    }
                }}
                <div class="absolute inset-0 flex items-center justify-center bg-base-100/80 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                    {edit_pencil_icon("w-4 h-4 text-primary")}
                </div>
            </button>
            <BaseModal
                open=open
                on_close=on_close
                container_class=Signal::derive(|| "w-full max-w-sm flex flex-col".to_string())
            >
                <div class="grid grid-cols-4 gap-2" role="group" aria-label="Version states">
                    {VersionState::VARIANTS.iter().map(|variant| {
                        let variant = *variant;
                        let is_current = Signal::derive(move || state.get() == variant);
                        let label = variant.label();
                        let color_class = variant.color_class();
                        view! {
                            <button
                                type="button"
                                class=move || {
                                    if variant == VersionState::Undefined {
                                        if is_current.get() {
                                            "btn btn-ghost rounded-none border border-base-content/30 text-base-content/50".to_string()
                                        } else {
                                            "btn btn-outline rounded-none border-white text-white".to_string()
                                        }
                                    } else if is_current.get() {
                                        "btn btn-outline rounded-none border-base-content/30 text-base-content/50".to_string()
                                    } else {
                                        format!(
                                            "btn btn-outline rounded-none {} {}",
                                            variant.border_class(),
                                            color_class
                                        )
                                    }
                                }
                                disabled=move || is_current.get()
                                aria-pressed=move || is_current.get().to_string()
                                on:click=move |_| {
                                    on_change.run(variant);
                                    open.set(false);
                                }
                            >
                                <span class=move || {
                                    if variant == VersionState::Undefined {
                                        if is_current.get() {
                                            "text-xs text-base-content/50".to_string()
                                        } else {
                                            "text-xs text-white".to_string()
                                        }
                                    } else if is_current.get() {
                                        "text-xs text-base-content/50".to_string()
                                    } else {
                                        format!("text-xs {color_class}")
                                    }
                                }>{label}</span>
                            </button>
                        }
                    }).collect_view()}
                </div>
            </BaseModal>
        </>
    }
}

/// Static badge that shows a version's state color to non-editors.
#[component]
fn VersionStateBadge(#[prop(into)] state: Signal<VersionState>) -> impl IntoView {
    view! {
        <div
            class=move || {
                let s = state.get();
                format!(
                    "w-10 h-10 rounded-none flex items-center justify-center bg-base-200/50 border {}",
                    s.border_class()
                )
            }
            aria-label=move || format!("State: {}", state.get().label())
        >
            {move || {
                let s = state.get();
                if s == VersionState::Undefined {
                    ifc_file_icon("w-5 h-5", s.color_class()).into_any()
                } else {
                    view! {
                        <span class=format!("text-sm font-semibold {}", s.color_class())>{s.letter()}</span>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

/// Add-new-version row placed at the top of the versions table in edit mode.
#[component]
fn AddVersionRow(
    #[prop(into)] is_uploading: Signal<bool>,
    #[prop(into)] on_click: Callback<()>,
) -> impl IntoView {
    view! {
        <tr class="border-b border-base-content/10 last:border-b-0">
            <td colspan="6" class="p-0">
                <button
                    type="button"
                    class=move || {
                        if is_uploading.get() {
                            "w-full rounded-none border border-dashed border-base-content/30 bg-base-200/30 p-4 flex items-center justify-center text-base-content/50 cursor-not-allowed"
                        } else {
                            "w-full rounded-none border border-dashed border-base-content/30 bg-base-200/30 p-4 flex items-center justify-center text-base-content/50 hover:border-primary hover:text-primary transition-colors cursor-pointer"
                        }
                    }
                    disabled=move || is_uploading.get()
                    on:click=move |_| on_click.run(())
                    aria-label="Add version"
                >
                    {move || if is_uploading.get() {
                        view! {
                            <span class="loading loading-spinner loading-sm" aria-hidden="true"></span>
                        }
                            .into_any()
                    } else {
                        view! {
                            <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                <line x1="12" y1="5" x2="12" y2="19" />
                                <line x1="5" y1="12" x2="19" y2="12" />
                            </svg>
                        }
                            .into_any()
                    }}
                </button>
            </td>
        </tr>
    }
}

/// Returns a tag's wire id.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn tag_id(tag: &Tag) -> String {
    tag.id().to_string()
}

/// Returns a tag's user-facing label.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn tag_label(tag: &Tag) -> String {
    tag.label().to_string()
}

/// Returns a tag's inline CSS color style.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn tag_color(tag: &Tag) -> String {
    tag.color().to_string()
}

#[component]
fn EditableChipSection<T: Clone + PartialEq + Send + Sync + 'static>(
    #[allow(unused_variables)] title: &'static str,
    #[allow(unused_variables)] aria_label: &'static str,
    #[allow(unused_variables)] items: Vec<String>,
    all_items: Vec<T>,
    editing: Signal<bool>,
    on_cancel: Callback<()>,
    on_toggle: Callback<String>,
    on_save: Callback<Vec<String>>,
    on_item_click: Callback<String>,
    id_fn: fn(&T) -> String,
    label_fn: fn(&T) -> String,
    color_fn: fn(&T) -> String,
    selected_items: Signal<Vec<String>>,
    badge_class: &'static str,
) -> impl IntoView {
    let all_items_for_label = all_items.clone();
    let all_items_for_color = all_items.clone();
    let resolve_label = move |id: &str| {
        all_items_for_label
            .iter()
            .find(|item| id_fn(item) == id)
            .map_or_else(|| id.to_owned(), label_fn)
    };
    let resolve_color = move |id: &str| {
        all_items_for_color
            .iter()
            .find(|item| id_fn(item) == id)
            .map_or_else(String::new, color_fn)
    };

    view! {
        <div class="space-y-3">
            <h3 class="text-sm font-semibold text-base-content">{title}</h3>
            {move || {
                if editing.get() {
                    let current_selected = selected_items.get();
                    view! {
                        <div class="space-y-2">
                            <div class="flex flex-wrap gap-2" role="group" aria-label=aria_label>
                                {all_items.iter().map(|item| {
                                    let item_id = id_fn(item);
                                    let item_id_for_class = item_id.clone();
                                    let item_id_for_aria = item_id.clone();
                                    let label = label_fn(item);
                                    view! {
                                        <button
                                            type="button"
                                            class=move || {
                                                let selected = selected_items.get().contains(&item_id_for_class);
                                                format!(
                                                    "badge badge-sm badge-outline rounded-none cursor-pointer transition-colors {}",
                                                    if selected {
                                                        "bg-primary/20 border-primary text-primary"
                                                    } else {
                                                        "border-base-content/20 text-base-content/70 hover:border-primary/50"
                                                    }
                                                )
                                            }
                                            on:click=move |_| on_toggle.run(item_id.clone())
                                            aria-pressed=move || selected_items.get().contains(&item_id_for_aria).to_string()
                                        >
                                            {label}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                            <div class="flex justify-end gap-2">
                                <button
                                    type="button"
                                    class="btn btn-ghost btn-xs"
                                    on:click=move |_| on_cancel.run(())
                                >"Cancel"</button>
                                <button
                                    type="button"
                                    class="btn btn-primary btn-xs"
                                    on:click=move |_| on_save.run(current_selected.clone())
                                >"Save"</button>
                            </div>
                        </div>
                    }
                        .into_any()
                } else {
                    view! {
                        <div class="flex flex-wrap gap-2" role="group" aria_label=aria_label>
                            {items.iter().map(|id| {
                                let id_for_click = id.clone();
                                let color = resolve_color(id);
                                let label = resolve_label(id);
                                view! {
                                    <button
                                        type="button"
                                        class=badge_class
                                        style=color
                                        on:click=move |_| on_item_click.run(id_for_click.clone())
                                    >
                                        {label}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

#[component]
fn ProjectModalContent(
    #[prop(into)] card: ProjectCardProperties,
    #[prop(into)] viewer_fullscreen: Signal<bool>,
    #[prop(into)] set_viewer_fullscreen: WriteSignal<bool>,
) -> impl IntoView {
    let current_user = CurrentUserContext::use_context();
    let projects_ctx = ProjectsContext::use_context();
    let modal = ProjectModalContext::use_context();
    let profile_modal = ProfileModalContext::use_context();
    let search = SearchContext::use_context();
    let metadata = MetadataContext::use_context();
    let is_editable = Signal::derive({
        let author_id = card.author_id.clone();
        let collaborator_ids = card.collaborator_ids.clone();
        move || {
            current_user.account.get().is_some_and(|me| {
                me.role == AccountRole::Admin
                    || me.id == author_id
                    || collaborator_ids.contains(&me.id)
            })
        }
    });

    let active_tab = modal.active_tab;
    let set_active_tab = modal.set_active_tab;

    let (editing_title, set_editing_title) = signal(false);
    let (draft_title, set_draft_title) = signal(card.title.clone());
    let (title, set_title) = signal(card.title.clone());

    let (editing_tags, set_editing_tags) = signal(false);
    let (draft_tags, set_draft_tags) = signal(card.tags.clone());
    let (tags, set_tags) = signal(card.tags.clone());

    let (editing_description, set_editing_description) = signal(false);
    let (draft_description, set_draft_description) = signal(card.description.clone());
    let (description, set_description) = signal(card.description.clone());

    let (editing_collaborators, set_editing_collaborators) = signal(false);
    let (collaborator_ids, set_collaborator_ids) = signal(card.collaborator_ids.clone());
    let (draft_collaborator_ids, set_draft_collaborator_ids) =
        signal(card.collaborator_ids.clone());

    let (versions, set_versions) = signal(Vec::<ProjectVersion>::new());
    let (versions_page, set_versions_page) = signal(0usize);
    let versions_total_pages = Signal::derive(move || {
        let total = versions.get().len();
        total.div_ceil(VERSIONS_PER_PAGE)
    });
    let clamped_versions_page = Signal::derive(move || {
        versions_page
            .get()
            .min(versions_total_pages.get().saturating_sub(1))
    });
    let (is_uploading_ifc, set_is_uploading_ifc) = signal(false);
    let (is_downloading, set_is_downloading) = signal(false);
    let (glb_status, set_glb_status) = signal(GlbConversionStatus::Idle);
    let (sidebar_collapsed, set_sidebar_collapsed) = signal(
        crate::utils::local_storage_get("project_modal.sidebar_collapsed")
            .is_some_and(|v| v == "true"),
    );

    Effect::new(move |_| {
        let collapsed = sidebar_collapsed.get();
        crate::utils::local_storage_set(
            "project_modal.sidebar_collapsed",
            if collapsed { "true" } else { "false" },
        );
    });

    let (is_mobile, set_is_mobile) = signal(
        leptos::web_sys::window()
            .and_then(|w| w.inner_width().ok())
            .and_then(|w| w.as_f64())
            .is_some_and(|width| width < 768.0),
    );

    Effect::new(move |_| {
        crate::utils::window_event_listener::<leptos::web_sys::Event, _>("resize", move |_| {
            let mobile = leptos::web_sys::window()
                .and_then(|w| w.inner_width().ok())
                .and_then(|w| w.as_f64())
                .is_some_and(|width| width < 768.0);
            set_is_mobile.set(mobile);
        });
    });

    Effect::new(move |_| {
        if is_mobile.get() && active_tab.get() == ProjectDetailsTab::Viewer3d {
            set_active_tab.set(ProjectDetailsTab::Versions);
        }
    });

    let project_id = card.id.clone();
    let viewer_ref = NodeRef::<leptos::html::Div>::new();

    Effect::new({
        let project_id = project_id.clone();
        move |_| {
            let project_id = project_id.clone();
            leptos::task::spawn_local(async move {
                match fetch_project_versions(&project_id).await {
                    Ok(fetched) => set_versions.set(fetched),
                    Err(err) => {
                        leptos::web_sys::console::error_1(
                            &format!("Failed to load project versions: {}", err.message()).into(),
                        );
                    }
                }
            });
        }
    });

    Effect::new({
        move |_| {
            crate::utils::document_event_listener::<web_sys::Event, _>(
                "fullscreenchange",
                move |_| {
                    let active = leptos::web_sys::window()
                        .and_then(|w| w.document())
                        .and_then(|d| d.fullscreen_element())
                        .is_some();
                    set_viewer_fullscreen.set(active);
                },
            );
        }
    });

    let toggle_fullscreen = Callback::new(move |()| {
        let active = leptos::web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.fullscreen_element())
            .is_some();
        if active {
            if let Some(document) = leptos::web_sys::window().and_then(|w| w.document()) {
                document.exit_fullscreen();
            }
            set_viewer_fullscreen.set(false);
        } else if let Some(element) = viewer_ref.get() {
            let _ = element.request_fullscreen();
        }
    });

    Effect::new(move |_| {
        crate::utils::window_event_listener::<leptos::web_sys::KeyboardEvent, _>("keydown", {
            let toggle_fullscreen = toggle_fullscreen;
            move |ev| {
                if active_tab.get() == ProjectDetailsTab::Viewer3d
                    && !is_mobile.get()
                    && ev.key().eq_ignore_ascii_case("f")
                    && !ev.ctrl_key()
                    && !ev.alt_key()
                    && !ev.meta_key()
                {
                    let target = ev
                        .target()
                        .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok());
                    if let Some(target) = target
                        && matches!(
                            target.tag_name().to_ascii_uppercase().as_str(),
                            "INPUT" | "TEXTAREA" | "SELECT"
                        )
                    {
                        return;
                    }
                    ev.prevent_default();
                    toggle_fullscreen.run(());
                }
            }
        });
    });

    let toast = ToastContext::use_context();
    let show_toast = move |message: String| toast.show(message);

    let (show_upload_modal, set_show_upload_modal) = signal(false);
    let (upload_file, set_upload_file) = signal(None::<web_sys::File>);
    let (upload_version, set_upload_version) = signal(String::new());
    let upload_modal_file_input = NodeRef::<leptos::html::Input>::new();

    let trigger_upload_modal = move || {
        set_upload_file.set(None);
        set_upload_version.set(String::new());
        set_show_upload_modal.set(true);
    };

    let close_upload_modal = move || {
        set_show_upload_modal.set(false);
        set_upload_file.set(None);
        set_upload_version.set(String::new());
    };

    let on_upload_file_selected = move |ev: leptos::web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<leptos::web_sys::HtmlInputElement>().ok())
        {
            if let Some(file) = input.files().and_then(|files| files.get(0)) {
                let is_ifc = std::path::Path::new(&file.name())
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("ifc"));
                if !is_ifc {
                    show_toast("IFC file must have a .ifc extension".to_string());
                    input.set_value("");
                    return;
                }
                set_upload_file.set(Some(file));
            }
            input.set_value("");
        }
    };

    let upload_ifc = {
        let project_id = project_id.clone();
        Callback::new(move |()| {
            let project_id = project_id.clone();
            leptos::task::spawn_local(async move {
                let Some(file) = upload_file.get_untracked() else {
                    show_toast("Select an IFC file to upload".to_string());
                    return;
                };
                let version = upload_version.get_untracked();
                let version = if version.trim().is_empty() {
                    "1.0.0".to_string()
                } else {
                    version
                };

                set_is_uploading_ifc.set(true);
                set_glb_status.set(GlbConversionStatus::Idle);
                match upload_project_ifc(&project_id, file, &version).await {
                    Ok(version) => {
                        let version_for_projects = version.clone();
                        set_versions.update(|versions| {
                            versions.insert(0, version);
                        });
                        projects_ctx.set_projects.update(|projects| {
                            for project in projects.iter_mut() {
                                if project.id == project_id {
                                    project.versions.insert(0, version_for_projects.clone());
                                    break;
                                }
                            }
                        });
                        modal.set_card.update(|opt| {
                            if let Some(card) = opt.as_mut() {
                                card.versions.insert(0, version_for_projects.clone());
                            }
                        });

                        // Trigger the IFC-to-GLB conversion and surface its status.
                        set_glb_status.set(GlbConversionStatus::Converting);
                        match convert_project_glb(&project_id).await {
                            Ok(true) => {
                                set_glb_status.set(GlbConversionStatus::Ready);
                                show_toast("IFC model converted to 3D".to_string());
                            }
                            Ok(false) => {
                                set_glb_status.set(GlbConversionStatus::NoGeometry);
                                show_toast("IFC model has no renderable geometry".to_string());
                            }
                            Err(err) => {
                                leptos::web_sys::console::error_1(
                                    &format!("Failed to convert IFC model: {}", err.message())
                                        .into(),
                                );
                                set_glb_status.set(GlbConversionStatus::Failed);
                                show_toast("Failed to convert IFC model".to_string());
                            }
                        }
                    }
                    Err(err) => {
                        leptos::web_sys::console::error_1(
                            &format!("Failed to upload IFC model: {}", err.message()).into(),
                        );
                        show_toast("Failed to upload IFC model".to_string());
                    }
                }
                set_is_uploading_ifc.set(false);
                close_upload_modal();
            });
        })
    };

    let (deleting_version_id, set_deleting_version_id) = signal(Option::<String>::None);

    let update_version_state = {
        let project_id = project_id.clone();
        Callback::new(move |(version_id, state): (String, VersionState)| {
            let project_id = project_id.clone();
            leptos::task::spawn_local(async move {
                match update_project_version_state(&project_id, &version_id, state).await {
                    Ok(()) => {
                        set_versions.update(|versions| {
                            for version in versions.iter_mut() {
                                if version.id == version_id {
                                    version.state = state;
                                    break;
                                }
                            }
                        });
                        projects_ctx.set_projects.update(|projects| {
                            for project in projects.iter_mut() {
                                if project.id == project_id {
                                    if let Some(version) =
                                        project.versions.iter_mut().find(|v| v.id == version_id)
                                    {
                                        version.state = state;
                                    }
                                    break;
                                }
                            }
                        });
                        modal.set_card.update(|opt| {
                            if let Some(card) = opt.as_mut()
                                && let Some(version) =
                                    card.versions.iter_mut().find(|v| v.id == version_id)
                            {
                                version.state = state;
                            }
                        });
                        show_toast("Version state updated".to_string());
                    }
                    Err(err) => {
                        leptos::web_sys::console::error_1(
                            &format!("Failed to update version state: {}", err.message()).into(),
                        );
                        show_toast("Failed to update version state".to_string());
                    }
                }
            });
        })
    };

    let delete_version = {
        let project_id = project_id.clone();
        Callback::new(move |version_id: String| {
            let project_id = project_id.clone();
            leptos::task::spawn_local(async move {
                set_deleting_version_id.set(Some(version_id.clone()));
                match delete_project_version(&project_id, &version_id).await {
                    Ok(()) => {
                        set_versions.update(|versions| {
                            versions.retain(|version| version.id != version_id);
                        });
                        if latest_visible_ifc_url(&versions.get_untracked()).is_none() {
                            set_glb_status.set(GlbConversionStatus::Idle);
                        }
                        projects_ctx.set_projects.update(|projects| {
                            for project in projects.iter_mut() {
                                if project.id == project_id {
                                    project.versions.retain(|v| v.id != version_id);
                                    break;
                                }
                            }
                        });
                        modal.set_card.update(|opt| {
                            if let Some(card) = opt.as_mut() {
                                card.versions.retain(|v| v.id != version_id);
                            }
                        });
                        show_toast("Version deleted".to_string());
                    }
                    Err(err) => {
                        leptos::web_sys::console::error_1(
                            &format!("Failed to delete version: {}", err.message()).into(),
                        );
                        show_toast("Failed to delete version".to_string());
                    }
                }
                set_deleting_version_id.set(None);
            });
        })
    };

    let (edit_mode, set_edit_mode) = signal(false);

    let (show_delete_confirm, set_show_delete_confirm) = signal(false);
    let (delete_confirm_input, set_delete_confirm_input) = signal(String::new());
    let (is_deleting, set_is_deleting) = signal(false);
    let can_delete =
        Signal::derive(move || delete_confirm_input.get().trim() == title.get().trim());

    let delete_project_click = {
        let project_id = project_id.clone();
        let set_projects = projects_ctx.set_projects;
        Callback::new(move |()| {
            let project_id = project_id.clone();
            let set_projects = set_projects;
            leptos::task::spawn_local(async move {
                set_is_deleting.set(true);
                match delete_project(&project_id).await {
                    Ok(()) => {
                        set_delete_confirm_input.set(String::new());
                        set_show_delete_confirm.set(false);
                        match fetch_projects().await {
                            Ok(refreshed) => set_projects.set(refreshed),
                            Err(err) => {
                                leptos::web_sys::console::error_1(
                                    &format!("Failed to refresh projects: {}", err.message())
                                        .into(),
                                );
                            }
                        }
                        modal.close();
                    }
                    Err(err) => {
                        leptos::web_sys::console::error_1(
                            &format!("Failed to delete project: {}", err.message()).into(),
                        );
                    }
                }
                set_is_deleting.set(false);
            });
        })
    };

    let start_edit_title = move || {
        set_draft_title.set(title.get());
        set_editing_title.set(true);
    };

    let cancel_edit_title = move || {
        set_draft_title.set(title.get());
        set_editing_title.set(false);
    };

    let start_edit_tags = move || {
        set_draft_tags.set(tags.get());
        set_editing_tags.set(true);
    };

    let cancel_edit_tags = move || {
        set_draft_tags.set(tags.get());
        set_editing_tags.set(false);
    };

    let start_edit_description = move || {
        set_draft_description.set(description.get());
        set_editing_description.set(true);
    };

    let cancel_edit_description = move || {
        set_draft_description.set(description.get());
        set_editing_description.set(false);
    };

    let start_edit_collaborators = move || {
        set_draft_collaborator_ids.set(collaborator_ids.get());
        set_editing_collaborators.set(true);
    };

    let cancel_edit_collaborators = move || {
        set_draft_collaborator_ids.set(collaborator_ids.get());
        set_editing_collaborators.set(false);
    };

    let toggle_edit_mode = move || {
        let next = !edit_mode.get();
        set_edit_mode.set(next);
        if !next {
            cancel_edit_title();
            cancel_edit_tags();
            cancel_edit_description();
            cancel_edit_collaborators();
        }
    };

    let commit_edit_title = {
        let project_id = project_id.clone();
        Callback::new(move |draft_value: String| {
            let project_id = project_id.clone();
            let set_title = set_title;
            let set_draft_title = set_draft_title;
            let set_editing_title = set_editing_title;
            let modal_card = modal.set_card;
            let set_projects = projects_ctx.set_projects;

            leptos::task::spawn_local(async move {
                match update_project_title(&project_id, draft_value).await {
                    Ok(new_title) => {
                        set_title.set(new_title.clone());
                        set_draft_title.set(new_title.clone());
                        modal_card.update(|opt| {
                            if let Some(card) = opt.as_mut() {
                                card.title.clone_from(&new_title);
                            }
                        });
                        set_projects.update(|projects| {
                            for project in projects.iter_mut() {
                                if project.id == project_id {
                                    project.title.clone_from(&new_title);
                                    break;
                                }
                            }
                        });
                    }
                    Err(err) => {
                        leptos::web_sys::console::error_1(
                            &format!("Failed to update title: {}", err.message()).into(),
                        );
                    }
                }
                set_editing_title.set(false);
            });
        })
    };

    let commit_edit_description = {
        let project_id = project_id.clone();
        Callback::new(move |draft_value: String| {
            let project_id = project_id.clone();
            let set_description = set_description;
            let set_draft_description = set_draft_description;
            let set_editing_description = set_editing_description;
            let modal_card = modal.set_card;
            let set_projects = projects_ctx.set_projects;

            leptos::task::spawn_local(async move {
                match update_project_description(&project_id, draft_value).await {
                    Ok(new_description) => {
                        set_description.set(new_description.clone());
                        set_draft_description.set(new_description.clone());
                        modal_card.update(|opt| {
                            if let Some(card) = opt.as_mut() {
                                card.description.clone_from(&new_description);
                            }
                        });
                        set_projects.update(|projects| {
                            for project in projects.iter_mut() {
                                if project.id == project_id {
                                    project.description.clone_from(&new_description);
                                    break;
                                }
                            }
                        });
                    }
                    Err(err) => {
                        leptos::web_sys::console::error_1(
                            &format!("Failed to update description: {}", err.message()).into(),
                        );
                    }
                }
                set_editing_description.set(false);
            });
        })
    };

    let commit_edit_tags = {
        let project_id = project_id.clone();
        Callback::new(move |draft_value: Vec<String>| {
            let project_id = project_id.clone();
            let set_tags = set_tags;
            let set_draft_tags = set_draft_tags;
            let set_editing_tags = set_editing_tags;
            let modal_card = modal.set_card;
            let set_projects = projects_ctx.set_projects;

            leptos::task::spawn_local(async move {
                match update_project_tags(&project_id, draft_value).await {
                    Ok(new_tags) => {
                        set_tags.set(new_tags.clone());
                        set_draft_tags.set(new_tags.clone());
                        modal_card.update(|opt| {
                            if let Some(card) = opt.as_mut() {
                                card.tags.clone_from(&new_tags);
                            }
                        });
                        set_projects.update(|projects| {
                            for project in projects.iter_mut() {
                                if project.id == project_id {
                                    project.tags.clone_from(&new_tags);
                                    break;
                                }
                            }
                        });
                    }
                    Err(err) => {
                        leptos::web_sys::console::error_1(
                            &format!("Failed to update tags: {}", err.message()).into(),
                        );
                    }
                }
                set_editing_tags.set(false);
            });
        })
    };

    let commit_edit_collaborators = {
        let project_id = project_id.clone();
        Callback::new(move |draft_value: Vec<String>| {
            let project_id = project_id.clone();
            let set_collaborator_ids = set_collaborator_ids;
            let set_draft_collaborator_ids = set_draft_collaborator_ids;
            let set_editing_collaborators = set_editing_collaborators;
            let modal_card = modal.set_card;
            let set_projects = projects_ctx.set_projects;

            leptos::task::spawn_local(async move {
                match update_project_collaborators(&project_id, draft_value).await {
                    Ok(new_ids) => {
                        set_collaborator_ids.set(new_ids.clone());
                        set_draft_collaborator_ids.set(new_ids.clone());
                        modal_card.update(|opt| {
                            if let Some(card) = opt.as_mut() {
                                card.collaborator_ids.clone_from(&new_ids);
                            }
                        });
                        set_projects.update(|projects| {
                            for project in projects.iter_mut() {
                                if project.id == project_id {
                                    project.collaborator_ids.clone_from(&new_ids);
                                    break;
                                }
                            }
                        });
                    }
                    Err(err) => {
                        leptos::web_sys::console::error_1(
                            &format!("Failed to update collaborators: {}", err.message()).into(),
                        );
                    }
                }
                set_editing_collaborators.set(false);
            });
        })
    };

    let toggle_favorite_click = {
        let project_id = card.id.clone();
        let set_projects = projects_ctx.set_projects;
        let modal_set_card = modal.set_card;
        Callback::new(move |()| {
            let project_id = project_id.clone();
            leptos::task::spawn_local(async move {
                match ProjectsContext::toggle_favorite(&project_id).await {
                    Ok(updated) => {
                        let updated_for_modal = updated.clone();
                        set_projects.update(|projects| {
                            if let Some(project) =
                                projects.iter_mut().find(|project| project.id == updated.id)
                            {
                                *project = updated.clone();
                            }
                        });
                        modal_set_card.update(|card| {
                            if let Some(card) = card.as_mut()
                                && card.id == updated_for_modal.id
                            {
                                card.favorites.clone_from(&updated_for_modal.favorites);
                            }
                        });
                    }
                    Err(err) => {
                        leptos::web_sys::console::error_1(
                            &format!("Failed to toggle favorite: {}", err.message()).into(),
                        );
                    }
                }
            });
        })
    };

    let is_favorited = Signal::derive({
        let project_id = project_id.clone();
        move || {
            projects_ctx
                .projects
                .get()
                .into_iter()
                .find(|project| project.id == project_id)
                .and_then(|project| {
                    current_user
                        .account
                        .get()
                        .map(|me| project.favorites.contains(&me.id))
                })
                .unwrap_or(false)
        }
    });
    let favorite_count = Signal::derive({
        let project_id = project_id.clone();
        move || {
            projects_ctx
                .projects
                .get()
                .into_iter()
                .find(|project| project.id == project_id)
                .map_or(card.favorites.len(), |project| project.favorites.len())
        }
    });

    let increment_downloads = {
        let project_id = card.id.clone();
        let set_projects = projects_ctx.set_projects;
        Callback::new(move |url: String| {
            if is_downloading.get_untracked() {
                return;
            }

            let project_id = project_id.clone();
            let set_is_downloading = set_is_downloading;
            let set_projects = set_projects;
            set_is_downloading.set(true);
            leptos::task::spawn_local(async move {
                match increment_project_downloads(&project_id).await {
                    Ok(updated) => {
                        trigger_download(&url);

                        set_projects.update(|projects| {
                            if let Some(project) =
                                projects.iter_mut().find(|project| project.id == updated.id)
                            {
                                *project = updated.clone();
                            }
                        });

                        match fetch_project_versions(&project_id).await {
                            Ok(fetched) => set_versions.set(fetched),
                            Err(err) => leptos::web_sys::console::error_1(
                                &format!(
                                    "Failed to refresh versions after download: {}",
                                    err.message()
                                )
                                .into(),
                            ),
                        }
                    }
                    Err(err) => {
                        leptos::web_sys::console::error_1(
                            &format!("Failed to increment downloads: {}", err.message()).into(),
                        );
                    }
                }
                gloo_timers::future::TimeoutFuture::new(1000).await;
                set_is_downloading.set(false);
            });
        })
    };

    let toggle_tag = Callback::new(move |tag: String| {
        set_draft_tags.update(|tags| {
            if let Some(pos) = tags.iter().position(|t| *t == tag) {
                tags.remove(pos);
            } else {
                tags.push(tag);
            }
        });
    });

    let add_collaborator = Callback::new(move |account_id: String| {
        set_draft_collaborator_ids.update(|ids| {
            if !ids.contains(&account_id) {
                ids.push(account_id);
            }
        });
    });

    let remove_collaborator = Callback::new(move |account_id: String| {
        set_draft_collaborator_ids.update(|ids| {
            if let Some(pos) = ids.iter().position(|id| id == &account_id) {
                ids.remove(pos);
            }
        });
    });

    let author_id = card.author_id.clone();
    let accounts = AccountsContext::use_context();

    let apply_filter = Callback::new(move |filter: String| {
        search.set_query.set(format!("#{filter}"));
        modal.close();
    });

    view! {
            <div class="flex flex-col h-full min-h-0 overflow-hidden gap-4">
                <BaseModal
                    open=Signal::derive(move || show_upload_modal.get())
                    on_close=Callback::new(move |()| close_upload_modal())
                    container_class=Signal::derive(|| "w-full max-w-lg flex flex-col".to_string())
                >
                    <div class="space-y-4">
                        <h3 class="text-sm font-semibold text-base-content">"Upload new version"</h3>
                        <input
                            type="file"
                            accept=".ifc"
                            class="hidden"
                            node_ref=upload_modal_file_input
                            on:change=on_upload_file_selected
                        />
                        <div class="space-y-2">
                            <label class="text-xs text-base-content/70 block">
                                "IFC file"
                                <span class="text-error ml-0.5" aria-hidden="true">"*"</span>
                            </label>
                            <button
                                type="button"
                                class=move || {
                                    if upload_file.get().is_some() {
                                        "w-full rounded-none border border-base-content/10 bg-base-200/30 p-2 text-left text-sm text-base-content"
                                    } else {
                                        "w-full rounded-none border border-dashed border-base-content/30 bg-base-200/30 p-2 text-left text-sm text-base-content/50 hover:border-primary hover:text-primary transition-colors"
                                    }
                                }
                                on:click=move |_| {
                                    if let Some(input) = upload_modal_file_input.get() {
                                        input.click();
                                    }
                                }
                            >
                                {move || upload_file.get().map_or_else(|| "Choose .ifc file".to_string(), |file| file.name())}
                            </button>
                        </div>
                        <div class="space-y-2">
                            <label class="text-xs text-base-content/70 block" for="upload-version-input">
                                "Version"
                                <span class="text-error ml-0.5" aria-hidden="true">"*"</span>
                            </label>
                            <input
                                id="upload-version-input"
                                type="text"
                                class="input input-sm input-bordered w-full rounded-none bg-transparent border-base-content/20 focus:border-primary focus:outline-none"
                                placeholder="1.0.0"
                                prop:value=move || upload_version.get()
                                on:input=move |ev| set_upload_version.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="flex justify-end gap-2">
                            <button
                                type="button"
                                class="btn btn-ghost btn-xs"
                                on:click=move |_| close_upload_modal()
                            >"Cancel"</button>
                            <button
                                type="button"
                                class=move || {
                                    let can_upload = upload_file.get().is_some()
                                        && !upload_version.get().trim().is_empty();
                                    if is_uploading_ifc.get() || !can_upload {
                                        "btn btn-primary btn-xs opacity-50 cursor-not-allowed"
                                    } else {
                                        "btn btn-primary btn-xs"
                                    }
                                }
                                disabled=move || {
                                    let can_upload = upload_file.get().is_some()
                                        && !upload_version.get().trim().is_empty();
                                    is_uploading_ifc.get() || !can_upload
                                }
                                on:click=move |_| upload_ifc.run(())
                            >
                                {move || if is_uploading_ifc.get() {
                                    view! {
                                        <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
                                    }
                                        .into_any()
                                } else {
                                    view! { "Upload" }.into_any()
                                }}
                            </button>
                        </div>
                    </div>
                </BaseModal>

                <div class=move || if viewer_fullscreen.get() { "flex flex-col min-h-0 overflow-hidden flex-1".to_string() } else { "flex flex-col min-h-0 overflow-hidden flex-1 py-2".to_string() }>
                    <div class=move || if viewer_fullscreen.get() { "flex-1 min-h-0".to_string() } else { "overflow-y-auto flex-1 min-h-0 p-2 pr-3".to_string() }>
                        <div class=move || {
                            if viewer_fullscreen.get() {
                                "grid grid-cols-1 gap-0 items-start h-full".to_string()
                            } else {
                                "flex flex-col xl:flex-row gap-6 items-start h-full".to_string()
                            }
                        }>
                            <div class=move || {
                                if viewer_fullscreen.get() {
                                    "min-w-0 h-full flex flex-col".to_string()
                                } else {
                                    "min-w-0 w-full flex-1 h-full flex flex-col rounded-none border border-base-content/10 bg-base-200/20 p-4".to_string()
                                }
                            }>
                                <div class=move || if viewer_fullscreen.get() { "hidden".to_string() } else { "flex items-center justify-between gap-3 pb-2 flex-shrink-0".to_string() }>
                                    <div class="tabs tabs-border">
                                        <button
                                            type="button"
                                            class=move || {
                                                if is_mobile.get() {
                                                    "tab tab-disabled opacity-50 cursor-not-allowed".to_string()
                                                } else if active_tab.get() == ProjectDetailsTab::Viewer3d {
                                                    "tab tab-active".to_string()
                                                } else {
                                                    "tab".to_string()
                                                }
                                            }
                                            disabled=move || is_mobile.get()
                                            on:click=move |_| {
                                                if !is_mobile.get() {
                                                    set_active_tab.set(ProjectDetailsTab::Viewer3d);
                                                }
                                            }
                                        >"3D viewer"</button>
                                        <button
                                            type="button"
                                            class=move || if active_tab.get() == ProjectDetailsTab::Versions { "tab tab-active" } else { "tab" }
                                            on:click=move |_| set_active_tab.set(ProjectDetailsTab::Versions)
                                        >"Versions"</button>
                                    </div>
                                </div>
                                <div class="flex-1 min-h-0">
                                    {move || match active_tab.get() {
                                        ProjectDetailsTab::Viewer3d => {
                                            let viewer_state = RwSignal::new(crate::components::ui::three_d_viewer::IfcViewerState::NoModel);
                                            let show_axes = RwSignal::new(load_axes_visible_from_preferences(
                                                current_user.account.get_untracked().as_ref(),
                                            ));

                                            Effect::new({
                                                let current_user = current_user;
                                                let set_show_axes = show_axes;
                                                move |_| {
                                                    let Some(account) = current_user.account.get() else {
                                                        return;
                                                    };
                                                    let new_visible =
                                                        load_axes_visible_from_preferences(Some(&account,
                                                        ));
                                                    if set_show_axes.get_untracked() != new_visible {
                                                        set_show_axes.set(new_visible);
                                                    }
                                                }
                                            });

                                            let initial_show_axes = show_axes.get_untracked();
                                            let axes_save_generation = Rc::new(std::cell::Cell::new(0u64));

                                            Effect::new({
                                                let current_user = current_user;
                                                let profile_modal = profile_modal;
                                                let axes_save_generation = Rc::clone(
                                                    &axes_save_generation);
                                                move |_| {
                                                    let visible = show_axes.get();
                                                    if visible == initial_show_axes {
                                                        return;
                                                    }
                                                    let Some(account) =
                                                        current_user.account.get_untracked()
                                                    else {
                                                        return;
                                                    };
                                                    let Some(new_preferences) =
                                                        preferences_with_axes_visible(
                                                            Some(&account),
                                                            visible,
                                                        )
                                                    else {
                                                        return;
                                                    };
                                                    if new_preferences == account.viewer_preferences {
                                                        return;
                                                    }

                                                    let expected = axes_save_generation
                                                        .get()
                                                        .wrapping_add(1);
                                                    axes_save_generation.set(expected);

                                                    let set_current_user = current_user.set_account;
                                                    let set_profile_account = profile_modal.set_account;
                                                    let axes_save_generation = Rc::clone(
                                                        &axes_save_generation);
                                                    leptos::task::spawn_local(async move {
                                                        gloo_timers::future::TimeoutFuture::new(300)
                                                            .await;
                                                        if axes_save_generation.get() != expected {
                                                            return;
                                                        }

                                                        match crate::contexts::current_user::update_viewer_preferences(
                                                            new_preferences,
                                                        )
                                                        .await
                                                        {
                                                            Ok(saved) => {
                                                                set_current_user.update(|opt| {
                                                                    if let Some(acc) = opt.as_mut() {
                                                                        acc.viewer_preferences
                                                                            .clone_from(
                                                                                &saved,
                                                                            );
                                                                    }
                                                                });
                                                                set_profile_account.update(|opt| {
                                                                    if let Some(acc) = opt.as_mut() {
                                                                        acc.viewer_preferences
                                                                            .clone_from(
                                                                                &saved,
                                                                            );
                                                                    }
                                                                });
                                                            }
                                                            Err(err) => {
                                                                leptos::web_sys::console::error_1(
                                                                    &format!(
                                                                        "Failed to save viewer preferences: {}",
                                                                        err.message()
                                                                    )
                                                                    .into(),
                                                                );
                                                            }
                                                        }
                                                    });
                                                }
                                            });
                                            let reset_view = RwSignal::new(false);
                                            let show_gizmo = RwSignal::new(load_gizmo_visible_from_preferences(
                                                current_user.account.get_untracked().as_ref(),
                                            ));

                                            Effect::new({
                                                let current_user = current_user;
                                                let set_show_gizmo = show_gizmo;
                                                move |_| {
                                                    let Some(account) = current_user.account.get() else {
                                                        return;
                                                    };
                                                    let new_visible =
                                                        load_gizmo_visible_from_preferences(Some(&account,
                                                        ));
                                                    if set_show_gizmo.get_untracked() != new_visible {
                                                        set_show_gizmo.set(new_visible);
                                                    }
                                                }
                                            });

                                            let initial_show_gizmo = show_gizmo.get_untracked();
                                            let visible_save_generation = Rc::new(std::cell::Cell::new(0u64));

                                            Effect::new({
                                                let current_user = current_user;
                                                let profile_modal = profile_modal;
                                                let visible_save_generation = Rc::clone(
    &visible_save_generation);
                                                move |_| {
                                                    let visible = show_gizmo.get();
                                                    if visible == initial_show_gizmo {
                                                        return;
                                                    }
                                                    let Some(account) =
                                                        current_user.account.get_untracked()
                                                    else {
                                                        return;
                                                    };
                                                    let Some(new_preferences) =
                                                        preferences_with_gizmo_visible(
                                                            Some(&account),
                                                            visible,
                                                        )
                                                    else {
                                                        return;
                                                    };
                                                    if new_preferences == account.viewer_preferences {
                                                        return;
                                                    }

                                                    let expected = visible_save_generation
                                                        .get()
                                                        .wrapping_add(1);
                                                    visible_save_generation.set(expected);

                                                    let set_current_user = current_user.set_account;
                                                    let set_profile_account = profile_modal.set_account;
                                                    let visible_save_generation = Rc::clone(
                                                        &visible_save_generation);
                                                    leptos::task::spawn_local(async move {
                                                        gloo_timers::future::TimeoutFuture::new(300)
                                                            .await;
                                                        if visible_save_generation.get() != expected {
                                                            return;
                                                        }

                                                        match crate::contexts::current_user::update_viewer_preferences(
                                                            new_preferences,
                                                        )
                                                        .await
                                                        {
                                                            Ok(saved) => {
                                                                set_current_user.update(|opt| {
                                                                    if let Some(acc) = opt.as_mut() {
                                                                        acc.viewer_preferences
                                                                            .clone_from(
                                                                                &saved,
                                                                            );
                                                                    }
                                                                });
                                                                set_profile_account.update(|opt| {
                                                                    if let Some(acc) = opt.as_mut() {
                                                                        acc.viewer_preferences
                                                                            .clone_from(
                                                                                &saved,
                                                                            );
                                                                    }
                                                                });
                                                            }
                                                            Err(err) => {
                                                                leptos::web_sys::console::error_1(
                                                                    &format!(
                                                                        "Failed to save viewer preferences: {}",
                                                                        err.message()
                                                                    )
                                                                    .into(),
                                                                );
                                                            }
                                                        }
                                                    });
                                                }
                                            });
                                            let gizmo_edit_mode = RwSignal::new(false);
                                            let gizmo_position = RwSignal::new(load_gizmo_position_from_preferences(
                                                current_user.account.get_untracked().as_ref(),
                                            ));
                                            let viewer_settings_open = RwSignal::new(false);

                                            Effect::new({
                                                let current_user = current_user;
                                                let set_gizmo_position = gizmo_position;
                                                move |_| {
                                                    let Some(account) = current_user.account.get() else {
                                                        return;
                                                    };
                                                    let new_pos =
                                                        load_gizmo_position_from_preferences(Some(&account,
                                                        ));
                                                    if set_gizmo_position.get_untracked() != new_pos {
                                                        set_gizmo_position.set(new_pos);
                                                    }
                                                }
                                            });

                                            let initial_gizmo_position = gizmo_position.get_untracked();
                                            let save_generation = Rc::new(std::cell::Cell::new(0u64));

                                            Effect::new({
                                                let current_user = current_user;
                                                let profile_modal = profile_modal;
                                                let save_generation = Rc::clone(&save_generation);
                                                move |_| {
                                                    let position = gizmo_position.get();
                                                    if position == initial_gizmo_position {
                                                        return;
                                                    }
                                                    let Some(account) =
                                                        current_user.account.get_untracked()
                                                    else {
                                                        return;
                                                    };
                                                    let Some(new_preferences) =
                                                        preferences_with_gizmo_position(
                                                            Some(&account),
                                                            position,
                                                        )
                                                    else {
                                                        return;
                                                    };
                                                    if new_preferences == account.viewer_preferences {
                                                        return;
                                                    }

                                                    let expected = save_generation.get().wrapping_add(1);
                                                    save_generation.set(expected);

                                                    let set_current_user = current_user.set_account;
                                                    let set_profile_account = profile_modal.set_account;
                                                    let save_generation = Rc::clone(&save_generation);
                                                    leptos::task::spawn_local(async move {
                                                        gloo_timers::future::TimeoutFuture::new(300)
                                                            .await;
                                                        if save_generation.get() != expected {
                                                            return;
                                                        }

                                                        match crate::contexts::current_user::update_viewer_preferences(
                                                            new_preferences,
                                                        )
                                                        .await
                                                        {
                                                            Ok(saved) => {
                                                                set_current_user.update(|opt| {
                                                                    if let Some(acc) = opt.as_mut() {
                                                                        acc.viewer_preferences
                                                                            .clone_from(
                                                                                &saved,
                                                                            );
                                                                    }
                                                                });
                                                                set_profile_account.update(|opt| {
                                                                    if let Some(acc) = opt.as_mut() {
                                                                        acc.viewer_preferences
                                                                            .clone_from(
                                                                                &saved,
                                                                            );
                                                                    }
                                                                });
                                                            }
                                                            Err(err) => {
                                                                leptos::web_sys::console::error_1(
                                                                    &format!(
                                                                        "Failed to save viewer preferences: {}",
                                                                        err.message()
                                                                    )
                                                                    .into(),
                                                                );
                                                            }
                                                        }
                                                    });
                                                }
                                            });

                                            let highlight_color = RwSignal::new(
                                                load_object_highlight_color_from_preferences(
                                                    current_user.account.get_untracked().as_ref(),
                                                ),
                                            );

                                            Effect::new({
                                                let current_user = current_user;
                                                let set_highlight_color = highlight_color;
                                                move |_| {
                                                    let Some(account) = current_user.account.get() else {
                                                        return;
                                                    };
                                                    let new_color =
                                                        load_object_highlight_color_from_preferences(
                                                            Some(&account),
                                                        );
                                                    if set_highlight_color.get_untracked() != new_color {
                                                        set_highlight_color.set(new_color);
                                                    }
                                                }
                                            });

                                            let initial_highlight_color = highlight_color.get_untracked();
                                            let highlight_save_generation =
                                                Rc::new(std::cell::Cell::new(0u64));

                                            Effect::new({
                                                let current_user = current_user;
                                                let profile_modal = profile_modal;
                                                let highlight_save_generation = Rc::clone(
                                                    &highlight_save_generation);
                                                move |_| {
                                                    let color = highlight_color.get();
                                                    if color == initial_highlight_color {
                                                        return;
                                                    }
                                                    let Some(account) =
                                                        current_user.account.get_untracked()
                                                    else {
                                                        return;
                                                    };
                                                    let Some(new_preferences) =
                                                        preferences_with_object_highlight_color(
                                                            Some(&account),
                                                            color,
                                                        )
                                                    else {
                                                        return;
                                                    };
                                                    if new_preferences == account.viewer_preferences {
                                                        return;
                                                    }

                                                    let expected = highlight_save_generation
                                                        .get()
                                                        .wrapping_add(1);
                                                    highlight_save_generation.set(expected);

                                                    let set_current_user = current_user.set_account;
                                                    let set_profile_account = profile_modal.set_account;
                                                    let highlight_save_generation = Rc::clone(
                                                        &highlight_save_generation);
                                                    leptos::task::spawn_local(async move {
                                                        gloo_timers::future::TimeoutFuture::new(300)
                                                            .await;
                                                        if highlight_save_generation.get() != expected {
                                                            return;
                                                        }

                                                        match crate::contexts::current_user::update_viewer_preferences(
                                                            new_preferences,
                                                        )
                                                        .await
                                                        {
                                                            Ok(saved) => {
                                                                set_current_user.update(|opt| {
                                                                    if let Some(acc) = opt.as_mut() {
                                                                        acc.viewer_preferences
                                                                            .clone_from(
                                                                                &saved,
                                                                            );
                                                                    }
                                                                });
                                                                set_profile_account.update(|opt| {
                                                                    if let Some(acc) = opt.as_mut() {
                                                                        acc.viewer_preferences
                                                                            .clone_from(
                                                                                &saved,
                                                                            );
                                                                    }
                                                                });
                                                            }
                                                            Err(err) => {
                                                                leptos::web_sys::console::error_1(
                                                                    &format!(
                                                                        "Failed to save viewer preferences: {}",
                                                                        err.message()
                                                                    )
                                                                    .into(),
                                                                );
                                                            }
                                                        }
                                                    });
                                                }
                                            });

                                            let skybox_color = RwSignal::new(
                                                load_skybox_color_from_preferences(
                                                    current_user.account.get_untracked().as_ref(),
                                                ),
                                            );

                                            Effect::new({
                                                let current_user = current_user;
                                                let set_skybox_color = skybox_color;
                                                move |_| {
                                                    let Some(account) = current_user.account.get() else {
                                                        return;
                                                    };
                                                    let new_color = load_skybox_color_from_preferences(
                                                        Some(&account),
                                                    );
                                                    if set_skybox_color.get_untracked() != new_color {
                                                        set_skybox_color.set(new_color);
                                                    }
                                                }
                                            });

                                            let initial_skybox_color = skybox_color.get_untracked();
                                            let skybox_save_generation =
                                                Rc::new(std::cell::Cell::new(0u64));

                                            Effect::new({
                                                let current_user = current_user;
                                                let profile_modal = profile_modal;
                                                let skybox_save_generation = Rc::clone(
                                                    &skybox_save_generation);
                                                move |_| {
                                                    let color = skybox_color.get();
                                                    if color == initial_skybox_color {
                                                        return;
                                                    }
                                                    let Some(account) =
                                                        current_user.account.get_untracked()
                                                    else {
                                                        return;
                                                    };
                                                    let Some(new_preferences) =
                                                        preferences_with_skybox_color(
                                                            Some(&account),
                                                            color,
                                                        )
                                                    else {
                                                        return;
                                                    };
                                                    if new_preferences == account.viewer_preferences {
                                                        return;
                                                    }

                                                    let expected = skybox_save_generation
                                                        .get()
                                                        .wrapping_add(1);
                                                    skybox_save_generation.set(expected);

                                                    let set_current_user = current_user.set_account;
                                                    let set_profile_account = profile_modal.set_account;
                                                    let skybox_save_generation = Rc::clone(
                                                        &skybox_save_generation);
                                                    leptos::task::spawn_local(async move {
                                                        gloo_timers::future::TimeoutFuture::new(300)
                                                            .await;
                                                        if skybox_save_generation.get() != expected {
                                                            return;
                                                        }

                                                        match crate::contexts::current_user::update_viewer_preferences(
                                                            new_preferences,
                                                        )
                                                        .await
                                                        {
                                                            Ok(saved) => {
                                                                set_current_user.update(|opt| {
                                                                    if let Some(acc) = opt.as_mut() {
                                                                        acc.viewer_preferences
                                                                            .clone_from(
                                                                                &saved,
                                                                            );
                                                                    }
                                                                });
                                                                set_profile_account.update(|opt| {
                                                                    if let Some(acc) = opt.as_mut() {
                                                                        acc.viewer_preferences
                                                                            .clone_from(
                                                                                &saved,
                                                                            );
                                                                    }
                                                                });
                                                            }
                                                            Err(err) => {
                                                                leptos::web_sys::console::error_1(
                                                                    &format!(
                                                                        "Failed to save viewer preferences: {}",
                                                                        err.message()
                                                                    )
                                                                    .into(),
                                                                );
                                                            }
                                                        }
                                                    });
                                                }
                                            });

                                            view! {
                                        <div node_ref=viewer_ref class="h-full flex flex-col">
                                            <div class="flex items-center justify-end gap-2 rounded-none border border-base-content/10 bg-base-200/30 p-2 flex-shrink-0">
                                                <div class="flex gap-1">
                                                    <ToolbarButton
                                                        label="Toggle axes gizmo"
                                                        tooltip_position=TooltipPosition::Left
                                                        on_click=Callback::new(move |()| {
                                                            show_axes.update(|v| *v = !*v);
                                                        })
                                                        disabled_overlay=Signal::derive(move || !show_axes.get())
                                                    >
                                                        <Icon::Axes />
                                                    </ToolbarButton>
                                                    <ToolbarButton
                                                        label="Reset view"
                                                        tooltip_position=TooltipPosition::Left
                                                        on_click=Callback::new(move |()| {
                                                            reset_view.set(true);
                                                        })
                                                        spin_on_click=true
                                                    >
                                                        <Icon::Reset />
                                                    </ToolbarButton>
                                                    <ToolbarButton
                                                        label="L+click to toggle view gizmo\nR+click to edit view gizmos position"
                                                        tooltip_position=TooltipPosition::Left
                                                        active=Signal::derive(move || gizmo_edit_mode.get())
                                                        on_click=Callback::new(move |()| {
                                                            show_gizmo.update(|v| *v = !*v);
                                                        })
                                                        on_context_menu=Callback::new(move |()| {
                                                            gizmo_edit_mode.update(|v| *v = !*v);
                                                        })
                                                        disabled_overlay=Signal::derive(move || !show_gizmo.get())
                                                    >
                                                        <Icon::Gizmo />
                                                    </ToolbarButton>
                                                    <ToolbarButton
                                                        label="Viewer settings"
                                                        tooltip_position=TooltipPosition::Left
                                                        on_click=Callback::new(move |()| {
                                                            viewer_settings_open.set(true);
                                                        })
                                                    >
                                                        <Icon::Gear />
                                                    </ToolbarButton>
                                                    <ToolbarButton
                                                        label="Toggle fullscreen"
                                                        tooltip_position=TooltipPosition::Left
                                                        on_click=toggle_fullscreen
                                                    >
                                                        {move || {
                                                            if viewer_fullscreen.get() {
                                                                view! { <Icon::FullscreenExit /> }
                                                                    .into_any()
                                                            } else {
                                                                view! { <Icon::FullscreenEnter /> }
                                                                    .into_any()
                                                            }
                                                        }}
                                                    </ToolbarButton>
                                                </div>
                                            </div>
                                            <div class="flex-1 min-h-0 relative">
                                                <IfcViewer
                                                    url=Signal::derive({
                                                        let project_id = project_id.clone();
                                                        move || {
                                                            latest_visible_ifc_url(&versions.get(),
                                                            ).map(|_| project_glb_url(&project_id))
                                                        }
                                                    })
                                                    metadata_url=Signal::derive({
                                                        let project_id = project_id.clone();
                                                        move || {
                                                            latest_visible_ifc_url(
                                                                &versions.get(),
                                                            )
                                                            .map(|_| project_glb_metadata_url(&project_id))
                                                        }
                                                    })
                                                    storage_key=Signal::derive({
                                                        let project_id = project_id.clone();
                                                        move || format!("cadiotheka.three_d_viewer.{project_id}")
                                                    })
                                                    state_signal=viewer_state
                                                    reset_view_signal=reset_view
                                                    show_axes_signal=show_axes
                                                    show_gizmo_signal=show_gizmo
                                                    gizmo_position_signal=gizmo_position
                                                    gizmo_edit_mode_signal=gizmo_edit_mode
                                                    highlight_color_signal=Signal::derive(move || highlight_color.get())
                                                    skybox_color_signal=Signal::derive(move || skybox_color.get())
                                                    disabled=Signal::derive({
                                                        let is_editable = is_editable;
                                                        let edit_mode = edit_mode;
                                                        move || is_editable.get() && edit_mode.get()
                                                    })
                                                    on_object_hit=Callback::new(move |hit: ObjectHit| {
                                                        leptos::web_sys::console::log_1(
                                                            &format!("Clicked object: {hit:?}").into(),
                                                        );
                                                    })
                                                />
                                            </div>
                                        </div>
                                    <ViewerSettingsModal
                                        open=Signal::derive(move || viewer_settings_open.get())
                                        on_close=Callback::new(move |()| viewer_settings_open.set(false))
                                        highlight_color=highlight_color
                                        skybox_color=skybox_color
                                    />
                                }.into_any()
                                        }
                                        ProjectDetailsTab::Versions => view! {
                                                <div class="min-h-0 h-full flex flex-col space-y-4 overflow-y-auto pr-1">
                                                    {move || match glb_status.get() {
                                                        GlbConversionStatus::Idle => ().into_any(),
                                                        GlbConversionStatus::Converting => view! {
                                                            <p class="text-xs text-primary flex items-center gap-1.5">
                                                                <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
                                                                <span>"Converting latest IFC to 3D model..."</span>
                                                            </p>
                                                        }.into_any(),
                                                        GlbConversionStatus::Ready => view! {
                                                            <p class="text-xs text-success flex items-center gap-1.5">"Latest IFC is ready to view in 3D"</p>
                                                        }.into_any(),
                                                        GlbConversionStatus::NoGeometry => view! {
                                                            <p class="text-xs text-warning">"Latest IFC has no renderable geometry"</p>
                                                        }.into_any(),
                                                        GlbConversionStatus::Failed => view! {
                                                            <p class="text-xs text-error">"Failed to convert latest IFC to 3D"</p>
                                                        }.into_any(),
                                                    }}
                                                    {move || {
                                                        let versions = versions.get();
                                                        let editing = is_editable.get() && edit_mode.get();
                                                        let add_row = move || {
                                                            if editing {
                                                                view! {
                                                                    <AddVersionRow
                                                                        is_uploading=Signal::derive(move || is_uploading_ifc.get())
                                                                        on_click=Callback::new(move |()| trigger_upload_modal())
                                                                    />
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                ().into_any()
                                                            }
                                                        };
                                                        if versions.is_empty() {
                                                            if editing {
                                                                view! {
                                                                    <div class="space-y-2">
                                                                        <div class="overflow-x-auto rounded-none border border-base-content/10">
                                                                            <table class="w-full text-left text-sm">
                                                                                <thead class="bg-base-200/50 text-xs uppercase text-base-content/70">
                                                                                    <tr>
                                                                                        <th class="p-2 w-10"></th>
                                                                                        <th class="p-2">"Version"</th>
                                                                                        <th class="p-2">"Type"</th>
                                                                                        <th class="p-2">"Published"</th>
                                                                                        <th class="p-2">"Downloads"</th>
                                                                                        <th class="p-2 w-10"></th>
                                                                                    </tr>
                                                                                </thead>
                                                                                <tbody>
                                                                                    {add_row()}
                                                                                </tbody>
                                                                            </table>
                                                                        </div>
                                                                        {move || {
                                                                            if versions_total_pages.get() <= 1 {
                                                                                ().into_any()
                                                                            } else {
                                                                                view! {
                                                                                    <div class="flex items-center justify-end text-sm">
                                                                                        <Pagination
                                                                                            page=versions_page
                                                                                            set_page=set_versions_page
                                                                                            total_pages=versions_total_pages
                                                                                        />
                                                                                    </div>
                                                                                }
                                                                                    .into_any()
                                                                            }
                                                                        }}
                                                                    </div>
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                view! {
                                                                    <div class="rounded-none border border-base-content/10 bg-base-200/30 p-8 text-center space-y-2">
                                                                        <p class="text-base-content/50 text-sm">"No IFC model uploaded yet."</p>
                                                                        <p class="text-base-content/40 text-xs">"Enter edit mode to add a version."</p>
                                                                    </div>
                                                                }
                                                                    .into_any()
                                                            }
                                                        } else {
                                                            let page = clamped_versions_page.get();
                                                            let start = page * VERSIONS_PER_PAGE;
                                                            let paginated: Vec<_> = versions
                                                                .into_iter()
                                                                .skip(start)
                                                                .take(VERSIONS_PER_PAGE)
                                                                .collect();
                                                            view! {
                                                                <div class="space-y-2">
                                                                    <div class="overflow-x-auto rounded-none border border-base-content/10">
                                                                        <table class="w-full text-left text-sm">
                                                                            <thead class="bg-base-200/50 text-xs uppercase text-base-content/70">
                                                                                <tr>
                                                                                    <th class="p-2 w-10"></th>
                                                                                    <th class="p-2">"Version"</th>
                                                                                    <th class="p-2">"Type"</th>
                                                                                    <th class="p-2">"Published"</th>
                                                                                    <th class="p-2">"Downloads"</th>
                                                                                    <th class="p-2 w-10"></th>
                                                                                </tr>
                                                                            </thead>
                                                                            <tbody>
                                                                                {add_row()}
                                                                                {paginated.into_iter().map(|version| {
                                                                                    let version_id = version.id.clone();
                                                                                    let version_id_for_state = version.id.clone();
                                                                                    let version_id_for_delete = version.id.clone();
                                                                                    let version_for_download = version.clone();
                                                                                    let is_deleting = Signal::derive({
                                                                                        let version_id = version_id.clone();
                                                                                        move || deleting_version_id.get().as_ref() == Some(&version_id)
                                                                                    });
                                                                                    let state = Signal::derive({
                                                                                        let version = version.clone();
                                                                                        move || version.state
                                                                                    });
                                                                                    view! {
                                                                                        <tr class="even:bg-base-200/30 border-b border-base-content/10 last:border-b-0">
                                                                                            <td class="p-2">
                                                                                                {move || {
                                                                                                    if is_editable.get() && edit_mode.get() {
                                                                                                        view! {
                                                                                                            <VersionStateSelector
                                                                                                                state=state
                                                                                                                on_change=Callback::new({
                                                                                                                    let version_id = version_id_for_state.clone();
                                                                                                                    move |new_state: VersionState| update_version_state.run((version_id.clone(), new_state))
                                                                                                                })
                                                                                                            />
                                                                                                        }
                                                                                                            .into_any()
                                                                                                    } else {
                                                                                                        view! { <VersionStateBadge state=state /> }.into_any()
                                                                                                    }
                                                                                                }}
                                                                                            </td>
                                                                                            <td class="p-2 font-medium text-base-content" title=version.filename.clone()>
                                                                                                {version.version.clone()}
                                                                                            </td>
                                                                                            <td class="p-2">
                                                                                                <span class="badge badge-sm badge-outline rounded-none border-primary text-primary whitespace-nowrap">
                                                                                                    "IFC"
                                                                                                </span>
                                                                                            </td>
                                                                                            <td class="p-2 text-base-content/70">{format_version_timestamp(&version.created_at)}</td>
                                                                                            <td class="p-2 text-base-content/70">{format_number(version.downloads.max(0).cast_unsigned())}</td>
                                                                                            <td class="p-2 text-right">
                                                                                                {move || {
                                                                                                    if is_editable.get() && edit_mode.get() {
                                                                                                        view! {
                                                                                                            <button
                                                                                                                type="button"
                                                                                                                class=move || {
                                                                                                                    if is_deleting.get() {
                                                                                                                        "btn btn-outline btn-error btn-xs p-1 h-auto min-h-0 opacity-50 cursor-not-allowed tooltip tooltip-left"
                                                                                                                    } else {
                                                                                                                        "btn btn-outline btn-error btn-xs p-1 h-auto min-h-0 tooltip tooltip-left"
                                                                                                                    }
                                                                                                                }
                                                                                                                disabled=move || is_deleting.get()
                                                                                                                on:click={
                                                                                                                    let version_id = version_id_for_delete.clone();
                                                                                                                    move |_| delete_version.run(version_id.clone())
                                                                                                                }
                                                                                                                aria-label="Delete version"
                                                                                                                data-tip="Delete"
                                                                                                            >
                                                                                                                {trash_icon("w-3.5 h-3.5")}
                                                                                                            </button>
                                                                                                        }
                                                                                                            .into_any()
                                                                                                    } else if version_for_download.state == VersionState::Undefined {
                                                                                                        ().into_any()
                                                                                                    } else {
                                                                                                        let version = version_for_download.clone();
                                                                                                        view! {
                                                                                                            <button
                                                                                                                type="button"
                                                                                                                class="btn btn-ghost btn-xs p-1 h-auto min-h-0 text-base-content/50 hover:text-primary tooltip tooltip-left"
                                                                                                                aria-label="Download"
                                                                                                                data-tip="Download"
                                                                                                                on:click=move |_| {
                                                                                                                    let url = ifc_download_url(&version);
                                                                                                                    increment_downloads.run(url);
                                                                                                                }
                                                                                                            >
                                                                                                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                                                                                                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                                                                                                                    <polyline points="7 10 12 15 17 10" />
                                                                                                                    <line x1="12" y1="15" x2="12" y2="3" />
                                                                                                                </svg>
                                                                                                            </button>
                                                                                                        }
                                                                                                            .into_any()
                                                                                                    }
                                                                                                }}
                                                                                            </td>
                                                                                        </tr>
                                                                                    }
                                                                                }).collect_view().into_any()}
                                                                            </tbody>
                                                                        </table>
                                                                    </div>
                                                                    {move || {
                                                                        if versions_total_pages.get() <= 1 {
                                                                            ().into_any()
                                                                        } else {
                                                                            view! {
                                                                                <div class="flex items-center justify-end text-sm">
                                                                                    <Pagination
                                                                                        page=versions_page
                                                                                        set_page=set_versions_page
                                                                                        total_pages=versions_total_pages
                                                                                    />
                                                                                </div>
                                                                            }
                                                                                .into_any()
                                                                        }
                                                                    }}
                                                                </div>
                                                            }
                                                                .into_any()
                                                        }
                                                    }}
                                                </div>
                                            }
                                                .into_any(),
                                    }}
                                </div>
                            </div>

                            <button
                                type="button"
                                class=move || {
                                    if viewer_fullscreen.get() {
                                        "hidden".to_string()
                                    } else {
                                        "hidden xl:flex self-stretch w-6 -mx-3 cursor-pointer group items-center justify-center".to_string()
                                    }
                                }
                                aria-label=move || if sidebar_collapsed.get() { "Expand sidebar" } else { "Collapse sidebar" }
                                on:click=move |_| set_sidebar_collapsed.update(|v| *v = !*v)
                            >
                                <div class="flex flex-col items-center justify-center w-full h-full">
                                    <div class="w-px flex-1 bg-base-content/10 group-hover:bg-primary transition-colors"></div>
                                    <div class="py-2 text-base-content/50 group-hover:text-primary transition-colors">
                                        {move || if sidebar_collapsed.get() { "<" } else { ">" }}
                                    </div>
                                    <div class="w-px flex-1 bg-base-content/10 group-hover:bg-primary transition-colors"></div>
                                </div>
                            </button>

                            <div class=move || {
                                if viewer_fullscreen.get() || sidebar_collapsed.get() {
                                    "hidden".to_string()
                                } else {
                                    "w-full xl:w-72 flex-shrink-0 space-y-4".to_string()
                                }
                            }>
                                {move || {
                                    if editing_title.get() {
                                        view! {
                                            <div class="rounded-none border border-base-content/10 bg-base-200/20 p-4 space-y-2">
                                                <div class="flex items-center gap-2">
                                                    <input
                                                        class=move || {
                                                            let at_max = draft_title.get().len() >= MAX_TITLE_LENGTH;
                                                            format!(
                                                                "input input-sm input-bordered flex-1 text-base-content text-lg font-bold {}",
                                                                if at_max { "hover:border-error" } else { "" }
                                                            )
                                                        }
                                                        type="text"
                                                        maxlength=MAX_TITLE_LENGTH.to_string()
                                                        prop:value=draft_title.get()
                                                        on:input=move |ev| set_draft_title.set(event_target_value(&ev))
                                                        on:keyup=move |ev| {
                                                            match ev.key().as_str() {
                                                                "Enter" => commit_edit_title.run(draft_title.get()),
                                                                "Escape" => cancel_edit_title(),
                                                                _ => {}
                                                            }
                                                        }
                                                        autofocus
                                                    />
                                                    <span class=move || {
                                                        if draft_title.get().len() >= MAX_TITLE_LENGTH {
                                                            "text-xs text-error flex-shrink-0"
                                                        } else {
                                                            "text-xs text-base-content/50 flex-shrink-0"
                                                        }
                                                    }>
                                                        {move || format!("{}/{}", draft_title.get().len(), MAX_TITLE_LENGTH)}
                                                    </span>
                                                </div>
                                                <div class="flex justify-end gap-2">
                                                    <button
                                                        type="button"
                                                        class="btn btn-ghost btn-xs"
                                                        on:click=move |_| cancel_edit_title()
                                                    >"Cancel"</button>
                                                    <button
                                                        type="button"
                                                        class="btn btn-primary btn-xs"
                                                        on:click=move |_| commit_edit_title.run(draft_title.get())
                                                    >"Save"</button>
                                                </div>
                                            </div>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <div class="rounded-none border border-base-content/10 bg-base-200/20 p-4">
                                                <div class="flex items-center gap-2">
                                                    <h2
                                                        class="text-lg font-bold text-primary leading-tight truncate tooltip tooltip-top"
                                                        data-tip={title.get()}
                                                    >
                                                        {title.get()}
                                                    </h2>
                                                    {move || (is_editable.get() && edit_mode.get()).then(|| view! {
                                                        <button
                                                            type="button"
                                                            class="btn btn-ghost btn-xs p-1 h-auto min-h-0 text-base-content/50 hover:text-primary"
                                                            aria-label="Edit title"
                                                            on:click=move |_| start_edit_title()
                                                        >
                                                                                    {edit_pencil_icon("w-4 h-4")}
                                                                                </button>
                                                    }.into_any())}
                                                </div>
                                            </div>
                                        }
                                            .into_any()
                                    }
                                }}

                                <div class="rounded-none border border-base-content/10 bg-base-200/20 p-4">
                                    <div class="flex flex-wrap items-center justify-between gap-2 text-xs">
                                        <div class="flex items-center gap-2">
                                            {
                                                move || {
                                                    let has_ifc = latest_visible_ifc_url(&versions.get()).is_some();
                                                    let downloading = is_downloading.get();
                                                    let editing = is_editable.get() && edit_mode.get();
                                                    let label = if downloading {
                                                        "Downloading IFC..."
                                                    } else if editing {
                                                        "Editing project"
                                                    } else if has_ifc {
                                                        "Download latest"
                                                    } else {
                                                        "No IFC model available"
                                                    };
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class=move || {
                                                                if !has_ifc || downloading || editing {
                                                                    "btn btn-ghost btn-xs p-1 h-auto min-h-0 text-base-content/30 cursor-not-allowed tooltip tooltip-bottom"
                                                                } else {
                                                                    "btn btn-ghost btn-xs p-1 h-auto min-h-0 text-base-content/50 hover:text-primary tooltip tooltip-bottom"
                                                                }
                                                            }
                                                            aria-label=label
                                                            data-tip=label
                                                            disabled=move || !has_ifc || downloading || editing
                                                            on:click={
                                                                let cb = increment_downloads;
                                                                move |_| {
                                                                    if let Some(url) = latest_visible_ifc_url(&versions.get()) {
                                                                        cb.run(url);
                                                                    }
                                                                }
                                                            }
                                                        >
                                                            <span class="flex items-center gap-1">
                                                                {if downloading {
                                                                    view! {
                                                                        <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <DownloadIcon /> }.into_any()
                                                                }}
                                                            </span>
                                                        </button>
                                                    }
                                                }
                                            }
                                            <button
                                                type="button"
                                                class=move || {
                                                    if is_favorited.get() {
                                                        "btn btn-ghost btn-xs p-1 h-auto min-h-0 text-error hover:text-base-content/50 tooltip tooltip-bottom"
                                                    } else {
                                                        "btn btn-ghost btn-xs p-1 h-auto min-h-0 text-base-content/50 hover:text-error tooltip tooltip-bottom"
                                                    }
                                                }
                                                aria-label=move || {
                                                    if is_favorited.get() {
                                                        format!("Remove {} from favorites", title.get())
                                                    } else {
                                                        format!("Add {} to favorites", title.get())
                                                    }
                                                }
                                                data-tip=move || {
                                                    if is_favorited.get() {
                                                        "Remove favorite".to_string()
                                                    } else {
                                                        "Add favorite".to_string()
                                                    }
                                                }
                                                on:click={
                                                    let cb = toggle_favorite_click;
                                                    move |_| cb.run(())
                                                }
                                            >
                                                <HeartIcon filled=Signal::derive(move || is_favorited.get()) />
                                                <span>{move || favorite_count.get().to_string()}</span>
                                            </button>
                                            {move || is_editable.get().then(|| view! {
                                                <button
                                                    type="button"
                                                    class=move || {
                                                        if edit_mode.get() {
                                                            "btn btn-ghost btn-xs p-1 h-auto min-h-0 text-primary tooltip tooltip-bottom"
                                                        } else {
                                                            "btn btn-ghost btn-xs p-1 h-auto min-h-0 text-base-content/50 hover:text-primary tooltip tooltip-bottom"
                                                        }
                                                    }
                                                    aria-label=move || if edit_mode.get() { "Leave edit mode" } else { "Enter edit mode" }
                                                    data-tip=move || if edit_mode.get() { "Done editing" } else { "Edit project" }
                                                    on:click=move |_| toggle_edit_mode()
                                                >
                                                    {edit_pencil_icon("w-4 h-4")}
                                                </button>
                                            }.into_any())}
                                        </div>
                                        <div class="flex items-center gap-2 text-base-content/50">
                                            <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">esc</kbd>
                                            <span>"to close"</span>
                                        </div>
                                    </div>
                                </div>

                                {move || {
                                    if editing_description.get() {
                                        view! {
                                            <div class="rounded-none border border-base-content/10 bg-base-200/20 p-4 space-y-3">
                                                <h3 class="text-sm font-semibold text-base-content">"About"</h3>
                                                <MarkdownEditor
                                                    value=draft_description
                                                    on_input=Callback::new(move |value| set_draft_description.set(value))
                                                    on_cancel=Callback::new(move |()| cancel_edit_description())
                                                    on_save=Callback::new(move |()| commit_edit_description.run(draft_description.get()))
                                                    maxlength=MAX_DESCRIPTION_LENGTH
                                                    editor_class="min-h-[12rem] font-mono text-sm"
                                                />
                                            </div>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            {move || {
                                                if is_editable.get() && edit_mode.get() {
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class="group relative text-left w-full rounded-none border border-base-content/10 bg-base-200/20 p-4 hover:border-primary transition-colors cursor-pointer"
                                                            aria-label="Edit description"
                                                            on:click=move |_| start_edit_description()
                                                        >
                                                            <span class="text-sm font-semibold text-base-content mb-2 block">"About"</span>
                                                            <div class="text-sm text-base-content/80 overflow-auto max-h-[12rem]">
                                                                <MarkdownView source=description.get() />
                                                            </div>
                                                            <div class="absolute inset-0 flex items-center justify-center bg-base-100/80 opacity-0 group-hover:opacity-100 transition-opacity">
                                                                {edit_pencil_icon("w-5 h-5 text-primary")}
                                                            </div>
                                                        </button>
                                                    }
                                                        .into_any()
                                                } else {
                                                    view! {
                                                        <div class="rounded-none border border-base-content/10 bg-base-200/20 p-4 space-y-2">
                                                            <h3 class="text-sm font-semibold text-base-content">"About"</h3>
                                                            <div class="text-sm text-base-content/80 overflow-auto max-h-[12rem]">
                                                                <MarkdownView source=description.get() />
                                                            </div>
                                                        </div>
                                                    }
                                                        .into_any()
                                                }
                                            }}
                                        }
                                            .into_any()
                                    }
                                }}

                                {move || {
                                    if editing_tags.get() {
                                        view! {
                                            <div class="rounded-none border border-base-content/10 bg-base-200/20 p-4">
                                                <EditableChipSection
                                                    title="Tags"
                                                    aria_label="Tags"
                                                    items=tags.get()
                                                    all_items={metadata.tags.get()}
                                                    editing=editing_tags.into()
                                                    on_cancel=Callback::new(move |()| cancel_edit_tags())
                                                    on_toggle=toggle_tag
                                                    on_save=Callback::new(move |selected| commit_edit_tags.run(selected))
                                                    on_item_click=Callback::new(move |id: String| {
                                                        let label = metadata
                                                            .tag_by_id(&id)
                                                            .map(|tag| tag.label().to_string())
                                                            .unwrap_or_default();
                                                        apply_filter.run(label);
                                                    })
                                                    id_fn=tag_id
                                                    label_fn=tag_label
                                                    color_fn=tag_color
                                                    selected_items=draft_tags.into()
                                                    badge_class="badge badge-sm badge-outline rounded-none text-neutral-900 border-base-content/10 whitespace-nowrap hover:border-primary/40 cursor-pointer"
                                                />
                                            </div>
                                        }
                                            .into_any()
                                    } else if is_editable.get() && edit_mode.get() {
                                        view! {
                                            <button
                                                type="button"
                                                class="group relative text-left w-full rounded-none border border-base-content/10 bg-base-200/20 p-4 hover:border-primary transition-colors cursor-pointer"
                                                aria-label="Edit tags"
                                                on:click=move |_| start_edit_tags()
                                            >
                                                <span class="text-sm font-semibold text-base-content mb-3 block">"Tags"</span>
                                                <div class="flex flex-wrap gap-2" role="group" aria-label="Tags">
                                                    {tags.get().iter().filter_map(|id| {
                                                        let tag = metadata.tag_by_id(id)?;
                                                        let style = tag.color().to_string();
                                                        let label = tag.label().to_string();
                                                        Some(view! {
                                                            <span
                                                                class="badge badge-sm badge-outline rounded-none text-neutral-900 border-base-content/10 whitespace-nowrap"
                                                                style=style
                                                            >
                                                                {label}
                                                            </span>
                                                        }.into_any())
                                                    }).collect_view()}
                                                </div>
                                                <div class="absolute inset-0 flex items-center justify-center bg-base-100/80 opacity-0 group-hover:opacity-100 transition-opacity">
                                                    {edit_pencil_icon("w-5 h-5 text-primary")}
                                                </div>
                                            </button>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <div class="rounded-none border border-base-content/10 bg-base-200/20 p-4">
                                                <h3 class="text-sm font-semibold text-base-content mb-3">"Tags"</h3>
                                                <div class="flex flex-wrap gap-2" role="group" aria-label="Tags">
                                                    {tags.get().iter().filter_map(|id| {
                                                        let tag = metadata.tag_by_id(id)?;
                                                        let style = tag.color().to_string();
                                                        let label = tag.label().to_string();
                                                        let label_click = label.clone();
                                                        Some(view! {
                                                            <button
                                                                type="button"
                                                                class="badge badge-sm badge-outline rounded-none text-neutral-900 border-base-content/10 whitespace-nowrap hover:border-primary/40 cursor-pointer"
                                                                style=style
                                                                on:click=move |_| apply_filter.run(label_click.clone())
                                                            >
                                                                {label}
                                                            </button>
                                                        }.into_any())
                                                    }).collect_view()}
                                                </div>
                                            </div>
                                        }
                                            .into_any()
                                    }
                                }}

                                {move || {
                                    let all_accounts = accounts.accounts.get();
                                    let author_id = author_id.clone();

                                    if editing_collaborators.get() {
                                        let owner_account = all_accounts.iter().find(|account| account.id == author_id).cloned();
                                        let all_accounts_for_select = all_accounts.clone();
                                        let (add_open, set_add_open) = signal(false);
                                        let add_open_signal = Signal::derive(move || add_open.get());
                                        let draft_query = RwSignal::new(String::new());
                                        let selectable_accounts = Memo::new(move |_| {
                                            let query = draft_query.get().to_lowercase();
                                            let excluded_ids: std::collections::HashSet<String> = std::iter::once(author_id.clone())
                                                .chain(draft_collaborator_ids.get().into_iter())
                                                .collect();
                                            all_accounts_for_select
                                                .clone()
                                                .into_iter()
                                                .filter(|account| !excluded_ids.contains(&account.id))
                                                .filter(|account| {
                                                    query.is_empty()
                                                        || account.username.to_lowercase().contains(&query)
                                                        || account.display_name.to_lowercase().contains(&query)
                                                })
                                                .collect::<Vec<_>>()
                                        });

                                        view! {
                                            <div class="rounded-none border border-base-content/10 bg-base-200/20 p-4 space-y-3">
                                                <h3 class="text-sm font-semibold text-base-content">"Authors"</h3>
                                                <div class="flex flex-wrap gap-2 items-center">
                                                    {owner_account.as_ref().map(|account| {
                                                        let account = account.clone();
                                                        view! {
                                                            {avatar_button(&account, None)}
                                                        }
                                                    })}
                                                    {draft_collaborator_ids.get().into_iter().filter_map(|id| {
                                                        let all_accounts = all_accounts.clone();
                                                        all_accounts.iter().find(|account| account.id == id).cloned()
                                                    }).map(|account| {
                                                        let account_id = account.id.clone();
                                                        let display_name = account.display_name.clone();
                                                        view! {
                                                            <div class="relative group">
                                                                {avatar_button(&account, None)}
                                                                <button
                                                                    type="button"
                                                                    class="absolute inset-0 flex items-center justify-center bg-error/80 opacity-0 group-hover:opacity-100 transition-opacity text-white tooltip tooltip-top"
                                                                    data-tip={format!("Remove {display_name}")}
                                                                    aria-label={format!("Remove {display_name}")}
                                                                    on:click=move |_| remove_collaborator.run(account_id.clone())
                                                                >
                                                                    <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                        <line x1="5" y1="12" x2="19" y2="12" />
                                                                    </svg>
                                                                </button>
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                    <button
                                                        type="button"
                                                        class="w-12 h-12 border border-dashed border-base-content/30 flex items-center justify-center text-base-content/50 hover:border-primary hover:text-primary transition-colors tooltip tooltip-top cursor-pointer"
                                                        aria-label="Add collaborator"
                                                        data-tip="Add collaborator"
                                                        on:click=move |_| set_add_open.set(true)
                                                    >
                                                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                            <line x1="12" y1="5" x2="12" y2="19" />
                                                            <line x1="5" y1="12" x2="19" y2="12" />
                                                        </svg>
                                                    </button>
                                                </div>
                                                <div class="flex justify-end gap-2">
                                                    <button
                                                        type="button"
                                                        class="btn btn-ghost btn-xs"
                                                        on:click=move |_| cancel_edit_collaborators()
                                                    >"Cancel"</button>
                                                    <button
                                                        type="button"
                                                        class="btn btn-primary btn-xs"
                                                        on:click=move |_| commit_edit_collaborators.run(draft_collaborator_ids.get())
                                                    >"Save"</button>
                                                </div>
                                                <BaseModal
                                                    open=add_open_signal
                                                    on_close=Callback::new(move |()| set_add_open.set(false))
                                                    container_class=Signal::derive(|| "w-full max-w-sm flex flex-col".to_string())
                                                >
                                                    <div class="space-y-3">
                                                        <h3 class="text-sm font-semibold text-base-content">"Add collaborator"</h3>
                                                        <input
                                                            type="text"
                                                            class="input w-full rounded-none bg-transparent border-base-content/20 focus:border-primary focus:outline-none"
                                                            placeholder="Search users..."
                                                            prop:value=draft_query.get()
                                                            on:input=move |ev| draft_query.set(event_target_value(&ev))
                                                        />
                                                        <div class="max-h-60 overflow-y-auto space-y-1">
                                                            {move || {
                                                                let accounts = selectable_accounts.get();
                                                                if accounts.is_empty() {
                                                                    view! {
                                                                        <p class="text-sm text-error py-2">"No users found."</p>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <div class="flex flex-wrap gap-2">
                                                                            {accounts.into_iter().map(|account| {
                                                                                let account_id = account.id.clone();
                                                                                let display_name = account.display_name.clone();
                                                                                view! {
                                                                                    <button
                                                                                        type="button"
                                                                                        class="flex items-center gap-2 px-2 py-1 border border-base-content/10 hover:border-primary/40 transition-colors"
                                                                                        on:click=move |_| {
                                                                                            add_collaborator.run(account_id.clone());
                                                                                            set_add_open.set(false);
                                                                                        }
                                                                                    >
                                                                                        {avatar_button(&account, Some("w-8 h-8".to_string()))}
                                                                                        <span class="text-sm text-base-content">{display_name.clone()}</span>
                                                                                        <span class="text-xs text-base-content/50">{format!("@{}", account.username)}</span>
                                                                                    </button>
                                                                                }
                                                                            }).collect_view()}
                                                                        </div>
                                                                    }.into_any()
                                                                }
                                                            }}
                                                        </div>
                                                    </div>
                                                </BaseModal>
                                            </div>
                                        }.into_any()
                                    } else {
                                        let owner_account = all_accounts.iter().find(|account| account.id == author_id).cloned();
                                        let current_collaborators = all_accounts
                                            .iter()
                                            .filter(|account| collaborator_ids.get().contains(&account.id))
                                            .cloned()
                                            .collect::<Vec<_>>();

                                        if is_editable.get() && edit_mode.get() {
                                            let collaborators = current_collaborators.clone();
                                            view! {
                                                <button
                                                    type="button"
                                                    class="group relative text-left w-full rounded-none border border-base-content/10 bg-base-200/20 p-4 hover:border-primary transition-colors cursor-pointer"
                                                    aria-label="Edit authors"
                                                    on:click=move |_| start_edit_collaborators()
                                                >
                                                    <span class="text-sm font-semibold text-base-content mb-3 block">"Authors"</span>
                                                    <div class="flex flex-wrap gap-2">
                                                        {owner_account.as_ref().map(|account| {
                                                            let account = account.clone();
                                                            view! {
                                                                {avatar_button(&account, Some("w-12 h-12 hover:border-primary".to_string()))}
                                                            }
                                                        })}
                                                        {collaborators.into_iter().map(|account| {
                                                            view! {
                                                                {avatar_button(&account, Some("w-12 h-12 hover:border-primary".to_string()))}
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                    <div class="absolute inset-0 flex items-center justify-center bg-base-100/80 opacity-0 group-hover:opacity-100 transition-opacity">
                                                        {edit_pencil_icon("w-5 h-5 text-primary")}
                                                    </div>
                                                </button>
                                            }.into_any()
                                        } else {
                                            let collaborators = current_collaborators.clone();
                                            view! {
                                                <div class="rounded-none border border-base-content/10 bg-base-200/20 p-4 space-y-3">
                                                    <h3 class="text-sm font-semibold text-base-content">"Authors"</h3>
                                                    <div class="flex flex-wrap gap-2">
                                                        {owner_account.as_ref().map(|account| {
                                                            let account_for_click = account.clone();
                                                            view! {
                                                                <button
                                                                    type="button"
                                                                    class="cursor-pointer"
                                                                    on:click=move |_| {
                                                                        profile_modal.open(account_for_click.clone());
                                                                    }
                                                                >
                                                                    {avatar_button(account, Some("w-12 h-12 hover:border-primary".to_string()))}
                                                                </button>
                                                            }
                                                        })}
                                                        {collaborators.into_iter().map(|account| {
                                                            let account_for_click = account.clone();
                                                            view! {
                                                                <button
                                                                    type="button"
                                                                    class="cursor-pointer"
                                                                    on:click=move |_| {
                                                                        profile_modal.open(account_for_click.clone());
                                                                    }
                                                                >
                                                                    {avatar_button(&account, Some("w-12 h-12 hover:border-primary".to_string()))}
                                                                </button>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                    }
                                }}

                                {move || {
                                    if is_editable.get() && edit_mode.get() {
                                        view! {
                                            <div class="border border-error/30 bg-error/10 p-4">
                                                {move || {
                                                    if show_delete_confirm.get() {
                                                        view! {
                                                            <div class="space-y-3">
                                                                <div class="flex items-start gap-3">
                                                                    {warning_icon("w-5 h-5 text-error flex-shrink-0 mt-0.5")}
                                                                    <div class="flex-1 min-w-0">
                                                                        <p class="text-sm font-semibold text-error">{"Danger zone"}</p>
                                                                        <p class="text-sm text-base-content/80">
                                                                            {"Deleting this project cannot be undone. Type "}
                                                                            <span class="font-semibold text-error">{title.get()}</span>
                                                                            {" to confirm."}
                                                                        </p>
                                                                    </div>
                                                                </div>
                                                                <input
                                                                    id="delete-confirm-input"
                                                                    type="text"
                                                                    class="input w-full rounded-none bg-transparent border-base-content/20 focus:border-error focus:outline-none"
                                                                    placeholder={title.get()}
                                                                    prop:value=delete_confirm_input.get()
                                                                    on:input=move |ev| set_delete_confirm_input.set(event_target_value(&ev))
                                                                    disabled=move || is_deleting.get()
                                                                />
                                                                <div class="flex justify-end gap-2">
                                                                    <button
                                                                        type="button"
                                                                        class="btn btn-ghost btn-xs"
                                                                        on:click=move |_| {
                                                                            set_show_delete_confirm.set(false);
                                                                            set_delete_confirm_input.set(String::new());
                                                                        }
                                                                        disabled=move || is_deleting.get()
                                                                    >
                                                                        {"Cancel"}
                                                                    </button>
                                                                    <button
                                                                        type="button"
                                                                        class="btn btn-error btn-xs"
                                                                        on:click=move |_| delete_project_click.run(())
                                                                        disabled=move || !can_delete.get() || is_deleting.get()
                                                                    >
                                                                        {move || if is_deleting.get() {
                                                                            view! {
                                                                                <span class="flex items-center gap-2">
                                                                                    <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
                                                                                    <span>{"Deleting..."}</span>
                                                                                </span>
                                                                            }
                                                                                .into_any()
                                                                        } else {
                                                                            view! {
                                                                                <span class="flex items-center gap-1">
                                                                                    {trash_icon("w-3.5 h-3.5")}
                                                                                    <span>{"Delete"}</span>
                                                                                </span>
                                                                            }
                                                                                .into_any()
                                                                        }}
                                                                    </button>
                                                                </div>
                                                            </div>
                                                        }
                                                            .into_any()
                                                    } else {
                                                        view! {
                                                            <button
                                                                type="button"
                                                                class="btn btn-outline btn-error w-full flex items-center justify-center gap-2"
                                                                on:click=move |_| set_show_delete_confirm.set(true)
                                                            >
                                                                {trash_icon("w-4 h-4")}
                                                                <span>{"Delete project"}</span>
                                                            </button>
                                                        }
                                                            .into_any()
                                                    }
                                                }}
                                            </div>
                                        }
                                            .into_any()
                                    } else {
                                        ().into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }
}

fn event_target_value(ev: &leptos::web_sys::Event) -> String {
    ev.target()
        .and_then(|t| {
            t.dyn_into::<leptos::web_sys::HtmlTextAreaElement>()
                .ok()
                .map(|textarea| textarea.value())
        })
        .or_else(|| {
            ev.target().and_then(|t| {
                t.dyn_into::<leptos::web_sys::HtmlInputElement>()
                    .ok()
                    .map(|input| input.value())
            })
        })
        .unwrap_or_default()
}

/// Programmatically starts a file download from the given URL.
///
/// A temporary anchor element with the `download` attribute is created and
/// clicked. The anchor is kept in the DOM for a short delay before removal,
/// because removing it immediately can cancel the download request before the
/// browser has initiated it.
fn trigger_download(url: &str) {
    let Some(document) = leptos::web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Ok(anchor) = document.create_element("a") else {
        return;
    };
    let filename = url.rsplit_once('/').map_or("model.ifc", |(_, name)| name);
    let _ = anchor.set_attribute("href", url);
    let _ = anchor.set_attribute("download", filename);
    let _ = anchor.set_attribute("style", "display:none");
    let _ = anchor.set_attribute("aria-hidden", "true");
    let Some(body) = document.body() else {
        return;
    };
    let _ = body.append_child(&anchor);
    if let Some(element) = anchor.dyn_ref::<leptos::web_sys::HtmlElement>() {
        element.click();
    }

    leptos::task::spawn_local(async move {
        gloo_timers::future::TimeoutFuture::new(1000).await;
        if let Some(parent) = anchor.parent_node() {
            let _ = parent.remove_child(&anchor);
        }
    });
}
