use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;

const DEFAULT_TOAST_DURATION_MS: u32 = 1500;

/// State for the single global toast notification.
#[derive(Clone, Copy)]
pub struct ToastContext {
    pub message: Signal<String>,
    pub visible: Signal<bool>,
    set_message: WriteSignal<String>,
    set_visible: WriteSignal<bool>,
}

impl ToastContext {
    /// Provide the global toast context.
    pub fn provide() {
        let (message, set_message) = signal(String::new());
        let (visible, set_visible) = signal(false);
        provide_context(Self {
            message: message.into(),
            visible: visible.into(),
            set_message,
            set_visible,
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }

    /// Show a toast with the given message for the default duration.
    pub fn show(&self, message: impl Into<String>) {
        self.set_message.set(message.into());
        self.set_visible.set(true);
        let set_visible = self.set_visible;
        leptos::task::spawn_local(async move {
            TimeoutFuture::new(DEFAULT_TOAST_DURATION_MS).await;
            set_visible.set(false);
        });
    }

    /// Dismiss the toast immediately.
    pub fn dismiss(&self) {
        self.set_visible.set(false);
    }
}
