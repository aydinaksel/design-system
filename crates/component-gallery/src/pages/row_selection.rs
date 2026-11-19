use std::collections::BTreeSet;

use design_system::formatting::format_money;
use design_system::table::{Column, DataTable, RowSelection, Search};
use design_system::{total, Currency, TotalsBar};
use leptos::prelude::*;

use super::section;

#[derive(Clone, Debug, PartialEq)]
struct Payment {
    reference: &'static str,
    payer: &'static str,
    received: &'static str,
    amount: f64,
}

fn payments() -> Vec<Payment> {
    [
        ("PAY-1041", "Harbour Stone Supply", "2026-11-02", 1840.0),
        ("PAY-1042", "Atlas Tile Works", "2026-11-03", 612.5),
        ("PAY-1043", "Coastline Interiors", "2026-11-03", 2975.0),
        ("PAY-1044", "Meridian Kitchens", "2026-11-05", 430.25),
        ("PAY-1045", "Northgate Builders", "2026-11-06", 5120.0),
        ("PAY-1046", "Juniper Design Studio", "2026-11-09", 88.0),
        ("PAY-1047", "Atlas Tile Works", "2026-11-10", 1299.99),
        ("PAY-1048", "Harbour Stone Supply", "2026-11-12", 760.0),
    ]
    .into_iter()
    .map(|(reference, payer, received, amount)| Payment {
        reference,
        payer,
        received,
        amount,
    })
    .collect()
}

fn payment_key(payment: &Payment) -> String {
    payment.reference.to_owned()
}

fn payment_columns() -> Vec<Column<Payment>> {
    vec![
        Column::text("reference", "Reference", |payment: &Payment| {
            payment.reference.to_owned()
        })
        .sortable(),
        Column::text("payer", "Payer", |payment: &Payment| {
            payment.payer.to_owned()
        })
        .sortable()
        .searchable()
        .filterable(),
        Column::text("received", "Received", |payment: &Payment| {
            payment.received.to_owned()
        })
        .sortable(),
        Column::money(
            "amount",
            "Amount",
            |payment: &Payment| Some(payment.amount),
            |_| Currency::UsDollar,
        )
        .sortable(),
    ]
}

fn picked_total(picked_keys: RwSignal<BTreeSet<String>>) -> f64 {
    picked_keys.with(|keys| {
        payments()
            .iter()
            .filter(|payment| keys.contains(payment.reference))
            .map(|payment| payment.amount)
            .sum()
    })
}

fn pick_one() -> AnyView {
    let picked_keys = RwSignal::new(BTreeSet::new());
    let rows = Signal::derive(|| Some(Ok(payments())));
    let picked = move || {
        picked_keys.with(|keys| {
            keys.first().map_or_else(
                || "Nothing picked yet.".to_owned(),
                |key| format!("Picked {key}."),
            )
        })
    };
    view! {
        <p class="mb-4 text-sm text-gray-600">
            "Click anywhere on a row to pick it. Picking another row replaces it."
        </p>
        <DataTable
            rows=rows
            columns=payment_columns()
            selection=RowSelection::single(picked_keys, payment_key)
        />
        <p class="mt-4 text-sm text-gray-700" aria-live="polite">
            {picked}
        </p>
    }
    .into_any()
}

fn pick_many() -> AnyView {
    let picked_keys = RwSignal::new(BTreeSet::new());
    let rows = Signal::derive(|| Some(Ok(payments())));
    let totals = move || {
        vec![total(
            "Picked",
            format_money(Some(picked_total(picked_keys)), Currency::UsDollar),
        )]
    };
    view! {
        <p class="mb-4 text-sm text-gray-600">
            "Hold Shift and click to pick every row between two clicks. The header checkbox picks every row in view, so search or filter first to narrow it."
        </p>
        <DataTable
            rows=rows
            columns=payment_columns()
            search=Search::InMemory
            search_placeholder="Search by Payer"
            selection=RowSelection::multiple(picked_keys, payment_key)
        />
        {move || view! { <TotalsBar totals=totals() /> }}
    }
    .into_any()
}

fn pick_rows_that_link() -> AnyView {
    let picked_keys = RwSignal::new(BTreeSet::new());
    let rows = Signal::derive(|| Some(Ok(payments())));
    view! {
        <p class="mb-4 text-sm text-gray-600">
            "When a row opens a page, clicking the row still opens it and only the checkbox picks it."
        </p>
        <DataTable
            rows=rows
            columns=payment_columns()
            row_href=std::sync::Arc::new(|_: &Payment| "/client-side-table".to_owned())
            sticky_first_column=true
            selection=RowSelection::multiple(picked_keys, payment_key)
        />
    }
    .into_any()
}

pub(crate) fn row_selection() -> AnyView {
    view! {
        <div class="space-y-8">
            {section("Pick One", pick_one())} {section("Pick Many", pick_many())}
            {section("Rows That Link", pick_rows_that_link())}
        </div>
    }
    .into_any()
}
