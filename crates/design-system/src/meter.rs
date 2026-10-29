use leptos::prelude::*;

use crate::tone::Tone;

const FULL_PERCENT: f64 = 100.0;

fn filled_percent(value: f64, maximum: f64) -> f64 {
    if maximum <= 0.0 || !value.is_finite() {
        return 0.0;
    }
    (value / maximum * FULL_PERCENT).clamp(0.0, FULL_PERCENT)
}

#[expect(
    clippy::must_use_candidate,
    reason = "the component macro drops attributes on the function it emits"
)]
#[component]
pub fn Meter(
    #[prop(into)] value: Signal<f64>,
    #[prop(default = FULL_PERCENT)] maximum: f64,
    #[prop(into)] label: String,
    #[prop(into, default = Signal::stored(Tone::Informative))] tone: Signal<Tone>,
) -> impl IntoView {
    view! {
        <div
            role="meter"
            aria-label=label
            aria-valuemin="0"
            aria-valuemax=maximum.to_string()
            aria-valuenow=move || value.get().to_string()
            class="h-2 w-full overflow-hidden rounded-full bg-gray-200"
        >
            <div
                class=move || {
                    format!(
                        "h-full rounded-full motion-safe:transition-[width] {}",
                        tone.get().fill_class(),
                    )
                }
                style=move || format!("width: {:.1}%", filled_percent(value.get(), maximum))
            ></div>
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_in_proportion_to_the_maximum() {
        assert!((filled_percent(30.0, 60.0) - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn never_overflows_or_goes_negative() {
        assert!((filled_percent(150.0, 100.0) - FULL_PERCENT).abs() < f64::EPSILON);
        assert!(filled_percent(-5.0, 100.0).abs() < f64::EPSILON);
        assert!(filled_percent(5.0, 0.0).abs() < f64::EPSILON);
        assert!(filled_percent(f64::NAN, 100.0).abs() < f64::EPSILON);
    }
}
