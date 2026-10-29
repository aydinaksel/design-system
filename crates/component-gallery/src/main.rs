#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use axum::Router;
    use component_gallery::app::{shell, App};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tracing::{event, Level};

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,hyper=warn".into()),
        )
        .init();

    let configuration = get_configuration(None)?;
    let leptos_options = configuration.leptos_options;
    let socket_address = leptos_options.site_addr;
    let route_list = generate_route_list(App);

    let router = Router::new()
        .leptos_routes(&leptos_options, route_list, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&socket_address).await?;
    event!(
        name: "server.start",
        Level::INFO,
        server.address = %socket_address,
        "component gallery listening on {{server.address}}",
    );
    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {}
