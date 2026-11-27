use std::fmt::Write as _;

use super::cell_value::Align;
use super::column::Column;

const TABLE_STYLE: &str =
    "border-collapse:collapse;font-family:Arial,Helvetica,sans-serif;font-size:13px;";
const HEADER_STYLE: &str =
    "border:1px solid #cccccc;padding:4px 8px;background:#f2f2f2;text-align:left;";
const CELL_STYLE: &str = "border:1px solid #cccccc;padding:4px 8px;";
const RIGHT_ALIGNED_STYLE: &str = "text-align:right;";
const CENTRED_STYLE: &str = "text-align:center;";
#[cfg(feature = "hydrate")]
const COPIED_MESSAGE: &str = "Copied Table to Clipboard";
#[cfg(feature = "hydrate")]
const COPY_FAILED_TITLE: &str = "Copy Failed";
#[cfg(feature = "hydrate")]
const COPY_FAILED_DESCRIPTION: &str = "The browser did not allow access to the clipboard.";

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn alignment_style(align: Align) -> &'static str {
    match align {
        Align::Left => "",
        Align::Right => RIGHT_ALIGNED_STYLE,
        Align::Center => CENTRED_STYLE,
    }
}

pub fn email_table_html<Row: Send + Sync + 'static>(
    columns: &[&Column<Row>],
    rows: &[Row],
    row_background: impl Fn(&Row) -> Option<&'static str>,
) -> String {
    let mut html = format!("<table style=\"{TABLE_STYLE}\"><thead><tr>");
    for column in columns {
        write!(
            html,
            "<th style=\"{HEADER_STYLE}\">{}</th>",
            escape_html(column.header)
        )
        .unwrap_or_default();
    }
    html.push_str("</tr></thead><tbody>");
    for row in rows {
        let background = row_background(row)
            .map(|colour| format!("background:{colour};"))
            .unwrap_or_default();
        html.push_str("<tr>");
        for column in columns {
            let text = column.plain_text_of(row).unwrap_or_default();
            write!(
                html,
                "<td style=\"{CELL_STYLE}{}{background}\">{}</td>",
                alignment_style(column.align),
                escape_html(&text)
            )
            .unwrap_or_default();
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody></table>");
    html
}

#[cfg_attr(
    not(feature = "hydrate"),
    expect(unused_variables, reason = "only a browser has a clipboard to copy to")
)]
pub fn copy_rows_for_email(html: &str) {
    #[cfg(feature = "hydrate")]
    match html_clipboard_write(html) {
        Some(write) => leptos::task::spawn_local(async move {
            if write.await.is_ok() {
                crate::toast::notify_success(COPIED_MESSAGE);
            } else {
                notify_copy_failed();
            }
        }),
        None => notify_copy_failed(),
    }
}

#[cfg(feature = "hydrate")]
fn notify_copy_failed() {
    crate::toast::notify_failure(COPY_FAILED_TITLE, Some(COPY_FAILED_DESCRIPTION.to_string()));
}

#[cfg(feature = "hydrate")]
fn html_clipboard_write(html: &str) -> Option<wasm_bindgen_futures::JsFuture> {
    use wasm_bindgen::JsValue;

    let window = web_sys::window()?;
    let parts = js_sys::Array::new();
    parts.push(&JsValue::from_str(html));
    let blob_options = web_sys::BlobPropertyBag::new();
    blob_options.set_type("text/html");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &blob_options).ok()?;
    let record = js_sys::Object::new();
    js_sys::Reflect::set(
        &record,
        &JsValue::from_str("text/html"),
        &js_sys::Promise::resolve(&blob),
    )
    .ok()?;
    let item = web_sys::ClipboardItem::new_with_record_from_str_to_blob_promise(&record).ok()?;
    let items = js_sys::Array::new();
    items.push(&item);
    Some(wasm_bindgen_futures::JsFuture::from(
        window.navigator().clipboard().write(&items),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::Currency;

    struct Line {
        item: String,
        amount: f64,
    }

    #[test]
    fn cells_carry_the_text_the_table_shows_escaped_and_aligned() {
        let item = Column::text("item", "Item", |line: &Line| Some(line.item.clone()));
        let amount = Column::money(
            "amount",
            "Amount",
            |line: &Line| Some(line.amount),
            |_| Currency::UsDollar,
        );
        let rows = [Line {
            item: "Tile <Honed>".to_owned(),
            amount: 1234.5,
        }];

        let html = email_table_html(&[&item, &amount], &rows, |_| Some("#fef2f2"));

        assert!(html.contains(">Item</th>"));
        assert!(html.contains("background:#fef2f2;\">Tile &lt;Honed&gt;</td>"));
        assert!(html.contains("text-align:right;background:#fef2f2;\">$1,234.50</td>"));
    }
}
