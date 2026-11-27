use leptos::prelude::*;
use leptos_meta::MetaTags;

pub fn shell<AppView, AppComponent>(
    options: LeptosOptions,
    stylesheet_href: &'static str,
    app: AppComponent,
) -> impl IntoView
where
    AppView: IntoView + 'static,
    AppComponent: Fn() -> AppView + Send + Sync + 'static,
{
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
                <link rel="stylesheet" id="leptos" href=stylesheet_href />
            </head>
            <body>{app()}</body>
        </html>
    }
}

pub const HYDRATED_ATTRIBUTE: &str = "data-hydrated";

#[cfg(feature = "hydrate")]
pub fn hydrate<AppView, AppComponent>(app: AppComponent)
where
    AppView: IntoView,
    AppComponent: FnOnce() -> AppView + 'static,
{
    hydrate_body(app);
    let marked = document()
        .document_element()
        .map(|root| root.set_attribute(HYDRATED_ATTRIBUTE, ""));
    if !matches!(marked, Some(Ok(()))) {
        leptos::logging::warn!("could not mark the document as hydrated");
    }
}
