use leptos::prelude::*;

use super::scale::{ValueFormat, ValueScale};

#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceLine {
    pub label: String,
    pub value: f64,
}

pub(crate) fn covered_maximum(data_maximum: f64, reference: Option<&ReferenceLine>) -> f64 {
    reference.map_or(data_maximum, |line| data_maximum.max(line.value))
}

pub(crate) fn reference_overlay(
    reference: Option<&ReferenceLine>,
    scale: &ValueScale,
    format: ValueFormat,
) -> AnyView {
    let Some(line) = reference else {
        return ().into_any();
    };
    let top_percent = scale.percent_from_top(line.value);
    view! {
        <div
            class="pointer-events-none absolute inset-x-0 z-[5] border-t-2 border-dashed border-gray-700"
            style=format!("top: {top_percent:.3}%")
        >
            <span class="absolute bottom-full left-0 mb-0.5 rounded-sm bg-surface/90 px-1 text-xs font-medium whitespace-nowrap text-gray-900">
                {format!("{} {}", line.label, format.compact(line.value))}
            </span>
        </div>
    }
    .into_any()
}
