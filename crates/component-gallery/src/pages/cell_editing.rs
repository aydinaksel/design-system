use std::sync::Arc;
use std::time::Duration;

use design_system::table::{CellEditor, Column, DataTable, RowAttribute};
use design_system::{FieldSave, Input, Switch};
use leptos::prelude::*;

const FAKE_SAVE_MILLISECONDS: u64 = 900;
const CONTAINER_PREFIX_LETTERS: usize = 4;
const CONTAINER_SERIAL_DIGITS: usize = 7;
const SERIAL_NUMBER_MAXIMUM_CHARACTERS: usize = 20;

#[derive(Clone, Debug, PartialEq)]
struct ShipmentLine {
    line_number: u32,
    item_code: &'static str,
    description: &'static str,
    container: String,
    serial_number: String,
}

fn shipment_lines() -> Vec<ShipmentLine> {
    vec![
        ShipmentLine {
            line_number: 1,
            item_code: "TRVT-024",
            description: "Ivory Honed Travertine 18x18",
            container: "CSQU3054383".to_owned(),
            serial_number: "SN-4410".to_owned(),
        },
        ShipmentLine {
            line_number: 2,
            item_code: "GRNT-076",
            description: "Black Polished Granite 12x24",
            container: String::new(),
            serial_number: String::new(),
        },
        ShipmentLine {
            line_number: 3,
            item_code: "SLTE-112",
            description: "Green Brushed Slate 16x16",
            container: "MSKU7219004".to_owned(),
            serial_number: "SN-4412".to_owned(),
        },
    ]
}

fn normalized_container(typed: &str) -> String {
    typed
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .to_uppercase()
}

fn container_error(container: &str) -> Option<String> {
    if container.is_empty() {
        return None;
    }
    let (prefix, serial): (Vec<char>, Vec<char>) = (
        container.chars().take(CONTAINER_PREFIX_LETTERS).collect(),
        container.chars().skip(CONTAINER_PREFIX_LETTERS).collect(),
    );
    let well_formed = prefix.len() == CONTAINER_PREFIX_LETTERS
        && prefix.iter().all(char::is_ascii_uppercase)
        && serial.len() == CONTAINER_SERIAL_DIGITS
        && serial.iter().all(char::is_ascii_digit);
    (!well_formed).then(|| {
        "A container number is four letters followed by seven digits, such as CSQU3054383."
            .to_owned()
    })
}

fn serial_number_error(serial_number: &str) -> Option<String> {
    (serial_number.trim().chars().count() > SERIAL_NUMBER_MAXIMUM_CHARACTERS).then(|| {
        format!("A serial number is at most {SERIAL_NUMBER_MAXIMUM_CHARACTERS} characters.")
    })
}

fn fake_save(save: FieldSave, fails: bool, commit: impl FnOnce() + 'static) {
    set_timeout(
        move || {
            if fails {
                save.failed("The server rejected the change: the line is locked by another user.");
                return;
            }
            commit();
            save.succeeded();
        },
        Duration::from_millis(FAKE_SAVE_MILLISECONDS),
    );
}

fn update_line(
    lines: RwSignal<Vec<ShipmentLine>>,
    line_number: u32,
    change: impl FnOnce(&mut ShipmentLine),
) {
    lines.update(|lines| {
        if let Some(line) = lines
            .iter_mut()
            .find(|line| line.line_number == line_number)
        {
            change(line);
        }
    });
}

fn container_column(
    lines: RwSignal<Vec<ShipmentLine>>,
    save_fails: RwSignal<bool>,
) -> Column<ShipmentLine> {
    Column::custom("container", "Container", move |line: &ShipmentLine| {
        let line_number = line.line_number;
        CellEditor::new(
            "Container",
            line.container.clone(),
            String::clone,
            |draft| view! { <Input value=draft /> }.into_any(),
            move |container: String, save: FieldSave| {
                fake_save(save, save_fails.get_untracked(), move || {
                    update_line(lines, line_number, |line| line.container = container);
                });
            },
        )
        .normalize(|typed: &String| normalized_container(typed))
        .validate(|container: &String| container_error(container))
        .hint(|draft| {
            let cleared = draft.with(|typed| typed.trim().is_empty());
            if cleared {
                "Saving an empty value clears the container number.".into_any()
            } else {
                "Spaces are removed and letters are capitalized.".into_any()
            }
        })
        .into_view()
    })
}

fn serial_number_column(
    lines: RwSignal<Vec<ShipmentLine>>,
    save_fails: RwSignal<bool>,
) -> Column<ShipmentLine> {
    Column::custom(
        "serial_number",
        "Serial Number",
        move |line: &ShipmentLine| {
            let line_number = line.line_number;
            CellEditor::text(
                "Serial Number",
                line.serial_number.clone(),
                move |serial_number: String, save: FieldSave| {
                    fake_save(save, save_fails.get_untracked(), move || {
                        update_line(lines, line_number, |line| {
                            line.serial_number = serial_number;
                        });
                    });
                },
            )
            .normalize(|typed: &String| typed.trim().to_owned())
            .validate(|serial_number: &String| serial_number_error(serial_number))
            .into_view()
        },
    )
}

pub(crate) fn cell_editing() -> AnyView {
    let lines = RwSignal::new(shipment_lines());
    let save_fails = RwSignal::new(false);
    let rows = Signal::derive(move || Some(Ok(lines.get())));
    let row_key: RowAttribute<ShipmentLine> =
        Arc::new(|line: &ShipmentLine| line.line_number.to_string());
    let columns = vec![
        Column::text("item", "Item", |line: &ShipmentLine| {
            line.item_code.to_owned()
        }),
        Column::text("description", "Description", |line: &ShipmentLine| {
            line.description.to_owned()
        }),
        container_column(lines, save_fails),
        serial_number_column(lines, save_fails),
    ];
    view! {
        <div class="mb-4 flex items-center gap-3 text-sm text-gray-700">
            <Switch
                checked=save_fails
                on_toggle=Callback::new(move |()| save_fails.update(|fails| *fails = !*fails))
                label="Make the Next Save Fail"
            />
            "Make the Next Save Fail"
        </div>
        <DataTable rows=rows columns=columns row_key=row_key />
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tidies_a_typed_container_number() {
        assert_eq!(normalized_container(" csqu 305 4383 "), "CSQU3054383");
    }

    #[test]
    fn accepts_a_well_formed_container_number() {
        assert_eq!(container_error("CSQU3054383"), None);
    }

    #[test]
    fn accepts_clearing_the_container_number() {
        assert_eq!(container_error(""), None);
    }

    #[test]
    fn rejects_a_container_number_with_too_few_digits() {
        assert!(container_error("CSQU305438").is_some());
    }
}
