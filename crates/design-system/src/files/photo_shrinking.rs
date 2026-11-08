#[cfg(feature = "hydrate")]
use leptos::wasm_bindgen::{closure::Closure, JsCast, JsValue};

#[cfg(feature = "hydrate")]
const SHRUNK_CONTENT_TYPE: &str = "image/jpeg";
#[cfg(feature = "hydrate")]
const SHRUNK_QUALITY: f64 = 0.85;
const SHRUNK_EXTENSION: &str = "jpg";

#[cfg_attr(
    all(not(feature = "hydrate"), not(test)),
    expect(dead_code, reason = "only a browser redraws photos")
)]
fn scaled_size(width: u32, height: u32, longest_edge_pixels: u32) -> Option<(u32, u32)> {
    let longest = width.max(height);
    if longest <= longest_edge_pixels {
        return None;
    }
    let scale = |edge: u32| {
        u64::from(edge)
            .checked_mul(u64::from(longest_edge_pixels))?
            .checked_div(u64::from(longest))
            .and_then(|scaled| u32::try_from(scaled).ok())
            .map(|scaled| scaled.max(1))
    };
    Some((scale(width)?, scale(height)?))
}

#[cfg_attr(
    all(not(feature = "hydrate"), not(test)),
    expect(dead_code, reason = "only a browser redraws photos")
)]
fn shrunk_name(name: &str) -> String {
    let stem = match name.rsplit_once('.') {
        Some((stem, _)) if !stem.is_empty() => stem,
        _ => name,
    };
    format!("{stem}.{SHRUNK_EXTENSION}")
}

#[cfg(feature = "hydrate")]
pub(super) async fn shrink_photo(file: web_sys::File, longest_edge_pixels: u32) -> web_sys::File {
    let Ok(address) = web_sys::Url::create_object_url_with_blob(&file) else {
        return file;
    };
    let shrunk = redraw_smaller(&file, &address, longest_edge_pixels).await;
    web_sys::Url::revoke_object_url(&address).unwrap_or_default();
    shrunk
        .filter(|smaller| smaller.size() < file.size())
        .unwrap_or(file)
}

#[cfg(not(feature = "hydrate"))]
pub(super) async fn shrink_photo(file: web_sys::File, _longest_edge_pixels: u32) -> web_sys::File {
    file
}

#[cfg(feature = "hydrate")]
async fn redraw_smaller(
    file: &web_sys::File,
    address: &str,
    longest_edge_pixels: u32,
) -> Option<web_sys::File> {
    let document = web_sys::window()?.document()?;
    let image: web_sys::HtmlImageElement = document.create_element("img").ok()?.dyn_into().ok()?;
    image.set_src(address);
    wasm_bindgen_futures::JsFuture::from(image.decode())
        .await
        .ok()?;
    let (width, height) = scaled_size(
        image.natural_width(),
        image.natural_height(),
        longest_edge_pixels,
    )?;
    let canvas: web_sys::HtmlCanvasElement =
        document.create_element("canvas").ok()?.dyn_into().ok()?;
    canvas.set_width(width);
    canvas.set_height(height);
    let context: web_sys::CanvasRenderingContext2d =
        canvas.get_context("2d").ok()??.dyn_into().ok()?;
    context
        .draw_image_with_html_image_element_and_dw_and_dh(
            &image,
            0.0,
            0.0,
            f64::from(width),
            f64::from(height),
        )
        .ok()?;
    let blob = encoded(&canvas).await?;
    let options = web_sys::FilePropertyBag::new();
    options.set_type(SHRUNK_CONTENT_TYPE);
    web_sys::File::new_with_blob_sequence_and_options(
        &js_sys::Array::of1(&blob),
        &shrunk_name(&file.name()),
        &options,
    )
    .ok()
}

#[cfg(feature = "hydrate")]
async fn encoded(canvas: &web_sys::HtmlCanvasElement) -> Option<web_sys::Blob> {
    let promise = js_sys::Promise::new(&mut |resolve: js_sys::Function, _reject| {
        let deliver = resolve.clone();
        let callback = Closure::once_into_js(move |blob: JsValue| {
            drop(deliver.call1(&JsValue::NULL, &blob));
        });
        let started = canvas.to_blob_with_type_and_encoder_options(
            callback.unchecked_ref(),
            SHRUNK_CONTENT_TYPE,
            &JsValue::from_f64(SHRUNK_QUALITY),
        );
        if started.is_err() {
            drop(resolve.call1(&JsValue::NULL, &JsValue::NULL));
        }
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .ok()?
        .dyn_into()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_photo_keeps_its_shape_while_its_longest_edge_shrinks() {
        assert_eq!(scaled_size(4032, 3024, 2048), Some((2048, 1536)));
        assert_eq!(scaled_size(3024, 4032, 2048), Some((1536, 2048)));
        assert_eq!(scaled_size(40_000, 10, 2048), Some((2048, 1)));
    }

    #[test]
    fn a_photo_already_small_enough_is_left_alone() {
        assert_eq!(scaled_size(2048, 1536, 2048), None);
        assert_eq!(scaled_size(0, 0, 2048), None);
    }

    #[test]
    fn a_shrunk_photo_is_named_for_its_new_format() {
        assert_eq!(shrunk_name("IMG_0412.HEIC"), "IMG_0412.jpg");
        assert_eq!(shrunk_name("crate.photo.jpeg"), "crate.photo.jpg");
        assert_eq!(shrunk_name("photo"), "photo.jpg");
        assert_eq!(shrunk_name(".jpg"), ".jpg.jpg");
    }
}
