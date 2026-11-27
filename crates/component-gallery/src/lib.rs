pub mod app;
pub mod contents;
pub mod deliberate_failure;
pub mod fake_data;
pub mod layout;
pub mod pages;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    design_system::hydrate(App);
}
