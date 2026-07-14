use leptos::prelude::*;
use std::time::Duration;

/// Spinner that only appears after a short delay to avoid flashes on fast networks.
#[component]
fn DelayedLoading() -> impl IntoView {
    let (show, set_show) = signal(false);

    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            gloo_timers::future::sleep(Duration::from_millis(300)).await;
            set_show.set(true);
        });
    });

    view! {
        {move || {
            show.get().then(|| {
                view! {
                    <div class="flex items-center justify-center text-center">
                        <span class="loading loading-spinner loading-lg text-primary"></span>
                    </div>
                }
            })
        }}
    }
}

pub fn data_grid<T, IV, F, R, E, EV>(
    resource: LocalResource<Result<Vec<T>, String>>,
    empty_message: E,
    render: F,
    on_retry: Option<R>,
    retry_label: &'static str,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
    IV: IntoView + 'static,
    F: Fn(Vec<T>) -> IV + Copy + Send + Sync + 'static,
    R: Fn() + Clone + Send + Sync + 'static,
    E: Fn() -> EV + Clone + Send + Sync + 'static,
    EV: IntoView + 'static,
{
    view! {
        <Suspense fallback=DelayedLoading>
            {move || match resource.get() {
                None => ().into_any(),
                Some(Err(err)) => {
                    let retry_view = on_retry.as_ref().map(|retry| {
                        let retry = retry.clone();
                        view! {
                            <button
                                class="btn btn-primary btn-sm mt-4"
                                on:click=move |_| retry()
                            >
                                {retry_label}
                            </button>
                        }
                    });
                    view! {
                        <div class="flex flex-col items-center justify-center text-center">
                            <span class="text-error">{err}</span>
                            {retry_view}
                        </div>
                    }
                        .into_any()
                }
                Some(Ok(data)) => {
                    if data.is_empty() {
                        view! {
                            <div class="flex items-center justify-center text-center">
                                <span class="text-base-content/70">{empty_message()}</span>
                            </div>
                        }
                            .into_any()
                    } else {
                        render(data).into_any()
                    }
                }
            }}
        </Suspense>
    }
}
