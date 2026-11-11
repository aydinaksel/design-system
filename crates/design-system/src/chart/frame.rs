use leptos::prelude::*;

use crate::typography::HeadingLevel;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ChartFrame {
    pub heading_level: HeadingLevel,
    pub labelled_by: String,
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ChartName {
    pub label: Option<String>,
    pub labelled_by: Option<String>,
    pub caption: String,
}

pub(crate) fn section_heading_level(chosen: Option<HeadingLevel>) -> HeadingLevel {
    chosen.unwrap_or_else(|| {
        use_context::<ChartFrame>().map_or(HeadingLevel::Three, |frame| frame.heading_level.below())
    })
}

pub(crate) fn chart_name(description: Option<String>) -> ChartName {
    if let Some(text) = description {
        return ChartName {
            label: Some(text.clone()),
            labelled_by: None,
            caption: text,
        };
    }
    use_context::<ChartFrame>().map_or_else(ChartName::default, |frame| ChartName {
        label: None,
        labelled_by: Some(frame.labelled_by),
        caption: frame.name,
    })
}
