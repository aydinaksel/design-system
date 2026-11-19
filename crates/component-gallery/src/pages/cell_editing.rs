use std::sync::Arc;
use std::time::Duration;

use design_system::table::{CellEditor, CellValue, Column, DataTable, RowAttribute};
use design_system::{
    ButtonTone, ConfirmDialog, DatePicker, FieldSave, Input, Listbox, ListboxOption, Switch,
};
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
    expected: String,
    status: String,
}

const STATUSES: [&str; 3] = ["Booked", "At Sea", "Arrived"];

fn shipment_lines() -> Vec<ShipmentLine> {
    vec![
        ShipmentLine {
            line_number: 1,
            item_code: "TRVT-024",
            description: "Ivory Honed Travertine 18x18",
            container: "CSQU3054383".to_owned(),
            serial_number: "SN-4410".to_owned(),
            expected: "2026-12-04".to_owned(),
            status: "At Sea".to_owned(),
        },
        ShipmentLine {
            line_number: 2,
            item_code: "GRNT-076",
            description: "Black Polished Granite 12x24",
            container: String::new(),
            serial_number: String::new(),
            expected: String::new(),
            status: "Booked".to_owned(),
        },
        ShipmentLine {
            line_number: 3,
            item_code: "SLTE-112",
            description: "Green Brushed Slate 16x16",
            container: "MSKU7219004".to_owned(),
            serial_number: "SN-4412".to_owned(),
            expected: "2026-11-20".to_owned(),
            status: "Arrived".to_owned(),
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

#[derive(Clone, Copy)]
struct ContainerRemoval {
    confirming: RwSignal<bool>,
    awaiting_answer: StoredValue<Option<(u32, FieldSave)>>,
}

impl ContainerRemoval {
    fn new() -> Self {
        let removal = Self {
            confirming: RwSignal::new(false),
            awaiting_answer: StoredValue::new(None),
        };
        Effect::new(move |_| {
            if removal.confirming.get() {
                return;
            }
            if let Some((_, save)) = removal.take() {
                save.invalid();
            }
        });
        removal
    }

    fn ask(self, line_number: u32, save: FieldSave) {
        self.awaiting_answer.set_value(Some((line_number, save)));
        self.confirming.set(true);
    }

    fn take(self) -> Option<(u32, FieldSave)> {
        self.awaiting_answer
            .try_update_value(Option::take)
            .flatten()
    }
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
    removal: ContainerRemoval,
) -> Column<ShipmentLine> {
    Column::view(
        "container",
        "Container",
        |line: &ShipmentLine| CellValue::optional_text(Some(&line.container)),
        move |line: &ShipmentLine| {
            let line_number = line.line_number;
            let has_container = !line.container.is_empty();
            CellEditor::new(
                "Container",
                line.container.clone(),
                String::clone,
                |draft| view! { <Input value=draft /> }.into_any(),
                move |container: String, save: FieldSave| {
                    if has_container && container.is_empty() {
                        removal.ask(line_number, save);
                        return;
                    }
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
                    "Saving an empty value clears the container number, after asking.".into_any()
                } else {
                    "Spaces are removed and letters are capitalized.".into_any()
                }
            })
            .into_view()
        },
    )
}

fn serial_number_column(
    lines: RwSignal<Vec<ShipmentLine>>,
    save_fails: RwSignal<bool>,
) -> Column<ShipmentLine> {
    Column::view(
        "serial_number",
        "Serial Number",
        |line: &ShipmentLine| CellValue::optional_text(Some(&line.serial_number)),
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

fn expected_column(
    lines: RwSignal<Vec<ShipmentLine>>,
    save_fails: RwSignal<bool>,
) -> Column<ShipmentLine> {
    Column::view(
        "expected",
        "Expected",
        |line: &ShipmentLine| CellValue::optional_text(Some(&line.expected)),
        move |line: &ShipmentLine| {
            let line_number = line.line_number;
            CellEditor::new(
                "Expected",
                line.expected.clone(),
                String::clone,
                |draft| view! { <DatePicker value=draft clearable=true /> }.into_any(),
                move |expected: String, save: FieldSave| {
                    fake_save(save, save_fails.get_untracked(), move || {
                        update_line(lines, line_number, |line| line.expected = expected);
                    });
                },
            )
            .into_view()
        },
    )
}

fn status_options() -> Vec<ListboxOption> {
    STATUSES
        .into_iter()
        .map(|status| ListboxOption::new(status, status))
        .collect()
}

fn status_column(
    lines: RwSignal<Vec<ShipmentLine>>,
    save_fails: RwSignal<bool>,
) -> Column<ShipmentLine> {
    Column::view(
        "status",
        "Status",
        |line: &ShipmentLine| CellValue::Text(line.status.clone()),
        move |line: &ShipmentLine| {
            let line_number = line.line_number;
            CellEditor::new(
                "Status",
                line.status.clone(),
                String::clone,
                |draft| view! { <Listbox options=status_options() selected=draft /> }.into_any(),
                move |status: String, save: FieldSave| {
                    fake_save(save, save_fails.get_untracked(), move || {
                        update_line(lines, line_number, |line| line.status = status);
                    });
                },
            )
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
    let removal = ContainerRemoval::new();
    let remove_container = move |()| {
        let Some((line_number, save)) = removal.take() else {
            return;
        };
        fake_save(save, save_fails.get_untracked(), move || {
            update_line(lines, line_number, |line| line.container = String::new());
        });
    };
    let columns = vec![
        Column::text("item", "Item", |line: &ShipmentLine| {
            line.item_code.to_owned()
        }),
        Column::text("description", "Description", |line: &ShipmentLine| {
            line.description.to_owned()
        }),
        container_column(lines, save_fails, removal),
        serial_number_column(lines, save_fails),
        expected_column(lines, save_fails),
        status_column(lines, save_fails),
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
        <ConfirmDialog
            open=removal.confirming
            title="Remove the Container Number?"
            body="The line will have no container until one is entered again."
            confirm_label="Remove Container"
            tone=ButtonTone::Danger
            on_confirm=Callback::new(remove_container)
        />
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
