use std::sync::Arc;
use std::time::Duration;

use design_system::table::{
    Align, CalculatedField, Column, LineCell, LineFix, LineForm, LineIssue, LineItems, LineSave,
    RowAttribute,
};
use design_system::{Input, InputKind, Listbox, ListboxOption};
use leptos::prelude::*;

const FAKE_SAVE_MILLISECONDS: u64 = 700;
const THOUSANDTHS_PER_UNIT: u64 = 1000;
const PENCE_PER_POUND: u64 = 100;
const DISCONTINUED_STONE: &str = "Discontinued Blue Limestone";
const STONES: [&str; 4] = [
    "Ivory Honed Travertine 18x18",
    "Black Polished Granite 12x24",
    "Green Brushed Slate 16x16",
    DISCONTINUED_STONE,
];

#[derive(Clone, Debug, PartialEq)]
struct StoneLine {
    letter: char,
    stone: String,
    quantity_thousandths: u64,
    area_per_crate_thousandths: u64,
    crates: u64,
    unit_price_pence: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct StoneLineDraft {
    stone: String,
    quantity: String,
    area_per_crate: String,
    crates: Option<String>,
    unit_price: String,
}

fn thousandths(text: &str) -> Option<u64> {
    let trimmed = text.trim();
    let (whole, fraction) = trimmed.split_once('.').unwrap_or((trimmed, ""));
    if fraction.len() > 3 || !fraction.chars().all(|digit| digit.is_ascii_digit()) {
        return None;
    }
    let padded = format!("{fraction:0<3}");
    whole
        .parse::<u64>()
        .ok()?
        .checked_mul(THOUSANDTHS_PER_UNIT)?
        .checked_add(padded.parse::<u64>().ok()?)
}

fn decimal_text(thousandths: u64) -> String {
    let whole = thousandths / THOUSANDTHS_PER_UNIT;
    let fraction = thousandths % THOUSANDTHS_PER_UNIT;
    if fraction == 0 {
        return whole.to_string();
    }
    format!("{whole}.{fraction:03}")
        .trim_end_matches('0')
        .to_owned()
}

fn pence(text: &str) -> Option<u64> {
    thousandths(text)
        .filter(|value| value % 10 == 0)
        .map(|value| value / 10)
}

fn pounds(pence: u64) -> String {
    format!(
        "£{}.{:02}",
        pence / PENCE_PER_POUND,
        pence % PENCE_PER_POUND
    )
}

fn positive_thousandths(text: &str) -> Option<u64> {
    thousandths(text).filter(|value| *value > 0)
}

fn quantity_fix(crates: u64, area_per_crate: u64) -> Option<LineFix<StoneLineDraft>> {
    let quantity = crates.checked_mul(area_per_crate)?;
    Some(LineFix {
        label: format!("Use {} m²", decimal_text(quantity)),
        apply: Arc::new(move |draft: &mut StoneLineDraft| {
            draft.quantity = decimal_text(quantity);
            draft.crates = None;
        }),
    })
}

fn crate_count_fix(crates: u64) -> LineFix<StoneLineDraft> {
    LineFix {
        label: format!("Count {crates} Crates"),
        apply: Arc::new(move |draft: &mut StoneLineDraft| {
            draft.crates = Some(crates.to_string());
        }),
    }
}

fn part_crate_issue(quantity: u64, area_per_crate: u64) -> LineIssue<StoneLineDraft> {
    let fewer = quantity.checked_div(area_per_crate).unwrap_or_default();
    let more = fewer.saturating_add(1);
    let fixes = [
        (fewer > 0)
            .then(|| quantity_fix(fewer, area_per_crate))
            .flatten(),
        quantity_fix(more, area_per_crate),
        Some(crate_count_fix(more)),
    ];
    LineIssue {
        field: "crates",
        message: format!(
            "{} m² does not fill a whole number of crates at {} m² per crate. The number of crates must be whole.",
            decimal_text(quantity),
            decimal_text(area_per_crate),
        ),
        fixes: fixes.into_iter().flatten().collect(),
    }
}

fn issue(field: &'static str, message: &str) -> LineIssue<StoneLineDraft> {
    LineIssue {
        field,
        message: message.to_owned(),
        fixes: Vec::new(),
    }
}

fn crates_issue(draft: &StoneLineDraft) -> Option<LineIssue<StoneLineDraft>> {
    if let Some(typed) = &draft.crates {
        let whole = typed
            .trim()
            .parse::<u64>()
            .ok()
            .filter(|crates| *crates > 0);
        return whole
            .is_none()
            .then(|| issue("crates", "Enter a whole number of crates above zero."));
    }
    let quantity = positive_thousandths(&draft.quantity)?;
    let area_per_crate = positive_thousandths(&draft.area_per_crate)?;
    (quantity.checked_rem(area_per_crate) != Some(0))
        .then(|| part_crate_issue(quantity, area_per_crate))
}

fn check(draft: &StoneLineDraft) -> Vec<LineIssue<StoneLineDraft>> {
    let stone = draft
        .stone
        .trim()
        .is_empty()
        .then(|| issue("stone", "Choose a stone."));
    let quantity = positive_thousandths(&draft.quantity).is_none().then(|| {
        issue(
            "quantity",
            "Enter a quantity above zero, to at most 3 decimal places.",
        )
    });
    let area = positive_thousandths(&draft.area_per_crate)
        .is_none()
        .then(|| issue("area_per_crate", "Enter the area each crate holds."));
    let price = (!draft.unit_price.trim().is_empty() && pence(&draft.unit_price).is_none())
        .then(|| issue("unit_price", "Enter a price in pounds and pence."));
    [stone, quantity, area, crates_issue(draft), price]
        .into_iter()
        .flatten()
        .collect()
}

fn calculated_crates(draft: &StoneLineDraft) -> Option<String> {
    let quantity = positive_thousandths(&draft.quantity)?;
    let area_per_crate = positive_thousandths(&draft.area_per_crate)?;
    let thousandths_of_crates = quantity
        .checked_mul(THOUSANDTHS_PER_UNIT)?
        .checked_div(area_per_crate)?;
    Some(decimal_text(thousandths_of_crates))
}

fn line_total_pence(quantity_thousandths: u64, unit_price_pence: u64) -> u64 {
    quantity_thousandths.saturating_mul(unit_price_pence) / THOUSANDTHS_PER_UNIT
}

fn stone_line(letter: char, draft: &StoneLineDraft) -> Option<StoneLine> {
    let quantity_thousandths = positive_thousandths(&draft.quantity)?;
    let area_per_crate_thousandths = positive_thousandths(&draft.area_per_crate)?;
    let crates = match &draft.crates {
        Some(typed) => typed.trim().parse().ok()?,
        None => quantity_thousandths.checked_div(area_per_crate_thousandths)?,
    };
    Some(StoneLine {
        letter,
        stone: draft.stone.clone(),
        quantity_thousandths,
        area_per_crate_thousandths,
        crates,
        unit_price_pence: pence(&draft.unit_price).unwrap_or_default(),
    })
}

fn draft_of(line: &StoneLine) -> StoneLineDraft {
    let calculated = line
        .quantity_thousandths
        .checked_div(line.area_per_crate_thousandths)
        .unwrap_or_default();
    let exact = line
        .quantity_thousandths
        .checked_rem(line.area_per_crate_thousandths)
        == Some(0);
    let overridden = !exact || calculated != line.crates;
    StoneLineDraft {
        stone: line.stone.clone(),
        quantity: decimal_text(line.quantity_thousandths),
        area_per_crate: decimal_text(line.area_per_crate_thousandths),
        crates: overridden.then(|| line.crates.to_string()),
        unit_price: format!(
            "{}.{:02}",
            line.unit_price_pence / PENCE_PER_POUND,
            line.unit_price_pence % PENCE_PER_POUND
        ),
    }
}

fn first_lines() -> Vec<StoneLine> {
    [
        ("Ivory Honed Travertine 18x18", "115.2", "9.6", "18.40"),
        ("Black Polished Granite 12x24", "48", "9.6", "22.10"),
    ]
    .into_iter()
    .zip('A'..)
    .filter_map(|((stone, quantity, area, price), letter)| {
        stone_line(
            letter,
            &StoneLineDraft {
                stone: stone.to_owned(),
                quantity: quantity.to_owned(),
                area_per_crate: area.to_owned(),
                crates: None,
                unit_price: price.to_owned(),
            },
        )
    })
    .collect()
}

fn line_cells(form: &LineForm<StoneLineDraft>) -> Vec<LineCell> {
    let draft = form.draft();
    let stone = form.bind(|line| line.stone.clone(), |line, text| line.stone = text);
    let quantity = form.bind(
        |line| line.quantity.clone(),
        |line, text| line.quantity = text,
    );
    let area = form.bind(
        |line| line.area_per_crate.clone(),
        |line, text| line.area_per_crate = text,
    );
    let crates = form.bind(
        |line| line.crates.clone().unwrap_or_default(),
        |line, text| {
            if line.crates.is_some() {
                line.crates = Some(text);
            }
        },
    );
    let price = form.bind(
        |line| line.unit_price.clone(),
        |line, text| line.unit_price = text,
    );
    let stone_options = Signal::stored(
        STONES
            .iter()
            .map(|stone| ListboxOption::new(*stone, *stone))
            .collect::<Vec<_>>(),
    );
    let crates_field = CalculatedField {
        calculated: Signal::derive(move || draft.with(calculated_crates)),
        overridden: Signal::derive(move || draft.with(|line| line.crates.is_some())),
        on_override: Callback::new(move |()| {
            draft.update(|line| line.crates = Some(calculated_crates(line).unwrap_or_default()));
        }),
        on_use_calculated: Callback::new(move |()| draft.update(|line| line.crates = None)),
    };
    let live_total = Signal::derive(move || {
        draft.with(|line| {
            positive_thousandths(&line.quantity)
                .zip(pence(&line.unit_price))
                .map(|(quantity, price)| pounds(line_total_pence(quantity, price)))
                .unwrap_or_default()
        })
    });
    vec![
        form.cell("stone", move || {
            view! { <Listbox options=stone_options selected=stone placeholder="Choose a Stone" /> }
        }),
        form.cell(
            "quantity",
            move || view! { <Input value=quantity kind=InputKind::Decimal /> },
        ),
        form.cell(
            "area_per_crate",
            move || view! { <Input value=area kind=InputKind::Decimal /> },
        ),
        form.calculated("crates", crates_field, move || {
            view! { <Input value=crates kind=InputKind::Integer /> }
        }),
        form.cell(
            "unit_price",
            move || view! { <Input value=price kind=InputKind::Decimal /> },
        ),
        form.shown("total", live_total),
    ]
}

fn fake_save(
    lines: RwSignal<Vec<StoneLine>>,
    key: Option<String>,
    draft: StoneLineDraft,
    save: LineSave<StoneLineDraft>,
) {
    set_timeout(
        move || {
            if draft.stone == DISCONTINUED_STONE {
                save.rejected(vec![issue(
                    "stone",
                    "The server refused this line: the stone is discontinued.",
                )]);
                return;
            }
            let letter = key
                .and_then(|key| key.chars().next())
                .unwrap_or_else(|| next_letter(lines));
            let Some(line) = stone_line(letter, &draft) else {
                save.failed("The server could not read the line.");
                return;
            };
            lines.update(
                |all| match all.iter_mut().find(|existing| existing.letter == letter) {
                    Some(existing) => *existing = line,
                    None => all.push(line),
                },
            );
            save.succeeded();
        },
        Duration::from_millis(FAKE_SAVE_MILLISECONDS),
    );
}

fn next_letter(lines: RwSignal<Vec<StoneLine>>) -> char {
    lines.with_untracked(|all| {
        ('A'..='Z')
            .find(|letter| all.iter().all(|line| line.letter != *letter))
            .unwrap_or('Z')
    })
}

fn total_pence(lines: &[StoneLine]) -> u64 {
    lines
        .iter()
        .map(|line| line_total_pence(line.quantity_thousandths, line.unit_price_pence))
        .sum()
}

fn columns() -> Vec<Column<StoneLine>> {
    vec![
        Column::text("letter", "Line", |line: &StoneLine| line.letter.to_string())
            .width_characters(4),
        Column::text("stone", "Stone", |line: &StoneLine| line.stone.clone()).width_characters(28),
        Column::text("quantity", "Quantity (m²)", |line: &StoneLine| {
            decimal_text(line.quantity_thousandths)
        })
        .align(Align::Right)
        .width_characters(12)
        .total(|lines| decimal_text(lines.iter().map(|line| line.quantity_thousandths).sum())),
        Column::text(
            "area_per_crate",
            "Area per Crate (m²)",
            |line: &StoneLine| decimal_text(line.area_per_crate_thousandths),
        )
        .align(Align::Right)
        .width_characters(12),
        Column::text("crates", "Crates", |line: &StoneLine| {
            line.crates.to_string()
        })
        .align(Align::Right)
        .width_characters(10)
        .total(|lines| {
            lines
                .iter()
                .map(|line| line.crates)
                .sum::<u64>()
                .to_string()
        }),
        Column::text("unit_price", "Unit Price", |line: &StoneLine| {
            pounds(line.unit_price_pence)
        })
        .align(Align::Right)
        .width_characters(10),
        Column::text("total", "Total", |line: &StoneLine| {
            pounds(line_total_pence(
                line.quantity_thousandths,
                line.unit_price_pence,
            ))
        })
        .align(Align::Right)
        .width_characters(12)
        .total(|lines| pounds(total_pence(lines))),
    ]
}

pub(crate) fn line_items() -> AnyView {
    let lines = RwSignal::new(first_lines());
    let rows = Signal::derive(move || Some(Ok(lines.get())));
    let row_key: RowAttribute<StoneLine> = Arc::new(|line: &StoneLine| line.letter.to_string());
    let on_save = Callback::new(
        move |(key, draft, save): (Option<String>, StoneLineDraft, LineSave<StoneLineDraft>)| {
            fake_save(lines, key, draft, save);
        },
    );
    let on_delete = Callback::new(move |key: String| {
        lines.update(|all| all.retain(|line| line.letter.to_string() != key));
    });
    view! {
        <LineItems
            rows=rows
            columns=columns()
            row_key=row_key
            draft_of=Arc::new(draft_of)
            new_draft=Arc::new(StoneLineDraft::default)
            form=Arc::new(line_cells)
            check=Arc::new(check)
            on_save=on_save
            on_delete=on_delete
        />
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(quantity: &str, area: &str) -> StoneLineDraft {
        StoneLineDraft {
            stone: STONES[0].to_owned(),
            quantity: quantity.to_owned(),
            area_per_crate: area.to_owned(),
            crates: None,
            unit_price: String::new(),
        }
    }

    #[test]
    fn reads_decimals_as_exact_thousandths() {
        assert_eq!(thousandths("115.2"), Some(115_200));
        assert_eq!(thousandths("9.6"), Some(9_600));
        assert_eq!(thousandths("1.2345"), None);
        assert_eq!(decimal_text(124_800), "124.8");
    }

    #[test]
    fn blocks_part_crates_and_offers_whole_ones() {
        let issues = check(&draft("120", "9.6"));
        let labels: Vec<String> = issues
            .iter()
            .flat_map(|issue| issue.fixes.iter().map(|fix| fix.label.clone()))
            .collect();
        assert_eq!(labels, ["Use 115.2 m²", "Use 124.8 m²", "Count 13 Crates"]);
        assert!(check(&draft("115.2", "9.6")).is_empty());
    }
}
