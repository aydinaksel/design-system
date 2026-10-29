use std::sync::Arc;

use leptos::prelude::*;

/// Everything a shared layout needs to render one company's chrome. Provide it
/// once per application with `provide_context`; the shared components read it
/// rather than branching on which site they are running inside.
#[derive(Clone, Default)]
pub struct Brand {
    pub name: &'static str,
    pub home_href: &'static str,
    pub home_link_label: &'static str,
    pub footer_note: &'static str,
    pub header_action: Option<Arc<dyn Fn() -> AnyView + Send + Sync>>,
}

impl Brand {
    #[must_use]
    pub fn with_header_action(
        mut self,
        header_action: impl Fn() -> AnyView + Send + Sync + 'static,
    ) -> Self {
        self.header_action = Some(Arc::new(header_action));
        self
    }
}

#[must_use]
pub fn brand() -> Brand {
    use_context::<Brand>().unwrap_or_default()
}
