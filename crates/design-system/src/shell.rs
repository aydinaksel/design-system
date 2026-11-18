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
