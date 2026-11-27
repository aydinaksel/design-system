pub const DELIBERATE_FAILURE_HEADER: &str = "x-deliberate-failure";
pub const DELIBERATE_FAILURE_VALUE: &str = "demonstrates an error state";

#[cfg(feature = "ssr")]
pub fn mark_response_as_deliberate_failure() {
    use axum::http::{HeaderName, HeaderValue};
    use leptos::prelude::use_context;
    use leptos_axum::ResponseOptions;

    if let Some(response) = use_context::<ResponseOptions>() {
        response.insert_header(
            HeaderName::from_static(DELIBERATE_FAILURE_HEADER),
            HeaderValue::from_static(DELIBERATE_FAILURE_VALUE),
        );
    }
}
