use leptos::ev::PointerEvent;
use leptos::html::Div;
use leptos::prelude::*;
use leptos_router::components::A;

use super::scale::ValueFormat;
use super::ChartColor;

const LONGEST_BAR_PERCENT: f64 = 82.0;
const ROW_GRID_CLASS: &str = "relative grid grid-cols-[minmax(0,11rem)_1fr] items-center gap-3";
const ROW_CLASS: &str = "group/bar block rounded-sm py-1 outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus";

#[derive(Clone, Debug, PartialEq)]
pub struct BarEntry {
    pub label: String,
    pub value: f64,
    pub href: Option<String>,
}

fn share_of(value: f64, total: f64) -> String {
    if total <= 0.0 {
        return String::new();
    }
    format!("{:.0}% of total", value / total * 100.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PointerOffset {
    pixels: f64,
    fraction: f64,
}

fn tooltip_style(offset: Option<PointerOffset>) -> String {
    offset.map_or_else(String::new, |PointerOffset { pixels, fraction }| {
        format!(
            "left: {pixels:.1}px; transform: translateX(-{:.1}%)",
            fraction * 100.0
        )
    })
}

fn bar_row(
    entry: BarEntry,
    fill_class: &'static str,
    longest: f64,
    total: f64,
    format: ValueFormat,
) -> AnyView {
    let width_percent = if longest > 0.0 {
        (entry.value / longest * LONGEST_BAR_PERCENT).max(0.0)
    } else {
        0.0
    };
    let share = share_of(entry.value, total);
    let row: NodeRef<Div> = NodeRef::new();
    let pointer: RwSignal<Option<PointerOffset>> = RwSignal::new(None);
    let on_pointer_move = move |event: PointerEvent| {
        let Some(element) = row.get_untracked() else {
            return;
        };
        let bounds = element.get_bounding_client_rect();
        if bounds.width() <= 0.0 {
            return;
        }
        let pixels = (f64::from(event.client_x()) - bounds.left()).clamp(0.0, bounds.width());
        pointer.set(Some(PointerOffset {
            pixels,
            fraction: pixels / bounds.width(),
        }));
    };
    let content = view! {
        <div node_ref=row class=ROW_GRID_CLASS>
            <span class="truncate text-sm text-gray-700" title=entry.label.clone()>
                {entry.label.clone()}
            </span>
            <span class="flex h-8 items-center">
                <span
                    class=format!(
                        "h-full rounded-r-[4px] transition-opacity hover:opacity-80 group-focus-visible/bar:opacity-80 {fill_class}",
                    )
                    style=format!("width: {width_percent:.3}%")
                    on:pointermove=on_pointer_move
                    on:pointerleave=move |_| pointer.set(None)
                ></span>
                <span class="ml-2 text-sm font-medium tabular-nums text-gray-900">
                    {format.compact(entry.value)}
                </span>
            </span>
            <span
                role="tooltip"
                class="pointer-events-none absolute bottom-full left-0 z-10 mb-1 space-y-0.5 rounded-item border border-gray-200 bg-surface px-3 py-2 text-xs whitespace-nowrap shadow-lg group-focus-visible/bar:block"
                class:hidden=move || pointer.get().is_none()
                style=move || tooltip_style(pointer.get())
            >
                <span class="block font-medium text-gray-900">{entry.label}</span>
                <span class="block font-semibold tabular-nums text-gray-900">
                    {format.full(entry.value)}
                </span>
                <span class="block text-gray-500">{share}</span>
            </span>
        </div>
    };
    match entry.href {
        Some(href) => view! {
            <A href=href scroll=false attr:class=format!("{ROW_CLASS} cursor-pointer")>
                {content}
            </A>
        }
        .into_any(),
        None => view! {
            <div class=ROW_CLASS tabindex="0">
                {content}
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn HorizontalBarChart(
    #[prop(into)] entries: Signal<Vec<BarEntry>>,
    #[prop(into)] description: String,
    #[prop(default = ChartColor::Blue)] color: ChartColor,
    #[prop(optional, into)] remainder: Signal<Option<BarEntry>>,
    format: ValueFormat,
    #[prop(default = "No data in this range.")] empty_message: &'static str,
) -> impl IntoView {
    let rows = move || {
        let mut bars = entries.get();
        let other = remainder.get();
        if bars.is_empty() {
            return view! { <p class="py-8 text-center text-sm text-gray-500">{empty_message}</p> }
                .into_any();
        }
        let total: f64 = bars
            .iter()
            .chain(other.iter())
            .map(|entry| entry.value)
            .sum();
        let longest = bars
            .iter()
            .chain(other.iter())
            .map(|entry| entry.value)
            .fold(0.0, f64::max);
        bars.sort_by(|left, right| right.value.total_cmp(&left.value));
        let ranked = bars
            .into_iter()
            .map(|entry| bar_row(entry, color.background_class(), longest, total, format))
            .collect_view();
        let remainder_row =
            other.map(|entry| bar_row(entry, "bg-gray-300", longest, total, format));
        view! {
            {ranked}
            {remainder_row}
        }
        .into_any()
    };
    view! {
        <figure aria-label=description class="space-y-1">
            {rows}
        </figure>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::share_of;

    #[test]
    fn share_rounds_to_a_whole_percent() {
        assert_eq!(share_of(1.0, 3.0), "33% of total");
    }

    #[test]
    fn an_empty_total_has_no_share() {
        assert_eq!(share_of(0.0, 0.0), "");
    }
}
