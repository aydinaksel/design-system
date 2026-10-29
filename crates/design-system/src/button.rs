#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonTone {
    Primary,
    #[default]
    Secondary,
    Danger,
    DangerOutline,
    Ghost,
}

impl ButtonTone {
    fn class(self) -> &'static str {
        match self {
            ButtonTone::Primary => {
                "border border-transparent bg-blue-600 text-white shadow-sm hover:bg-blue-700 focus-visible:ring-blue-500"
            }
            ButtonTone::Secondary => {
                "border border-gray-300 bg-white text-gray-700 shadow-sm hover:bg-gray-50 focus-visible:ring-gray-400"
            }
            ButtonTone::Danger => {
                "border border-transparent bg-red-600 text-white shadow-sm hover:bg-red-700 focus-visible:ring-red-500"
            }
            ButtonTone::DangerOutline => {
                "border border-red-300 bg-white text-red-600 shadow-sm hover:bg-red-50 focus-visible:ring-red-400"
            }
            ButtonTone::Ghost => {
                "border border-transparent bg-transparent text-gray-400 hover:bg-gray-100 hover:text-gray-600 focus-visible:ring-gray-400"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[default]
    Standard,
    Compact,
    Dense,
}

impl ButtonSize {
    pub(crate) fn class(self) -> &'static str {
        match self {
            ButtonSize::Standard => "gap-1.5 px-4 py-2 text-sm",
            ButtonSize::Compact => "gap-1.5 px-3 py-1.5 text-sm",
            ButtonSize::Dense => "gap-1 px-2.5 py-1 text-xs",
        }
    }

    fn square_class(self) -> &'static str {
        match self {
            ButtonSize::Standard => "size-[2.375rem] shrink-0 text-sm",
            ButtonSize::Compact => "size-[2.125rem] shrink-0 text-sm",
            ButtonSize::Dense => "size-[1.625rem] shrink-0 text-xs",
        }
    }
}

pub(crate) const SHARED_CLASS: &str = "inline-flex items-center justify-center whitespace-nowrap rounded-md font-medium transition-colors cursor-pointer focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-1 disabled:opacity-60 disabled:cursor-not-allowed";

#[must_use]
pub fn button_class(tone: ButtonTone, size: ButtonSize) -> String {
    format!("{SHARED_CLASS} {} {}", size.class(), tone.class())
}

#[must_use]
pub fn icon_button_class(tone: ButtonTone, size: ButtonSize) -> String {
    format!("{SHARED_CLASS} {} {}", size.square_class(), tone.class())
}

#[must_use]
pub fn button_icon_class(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::Standard | ButtonSize::Compact => "h-4 w-4 shrink-0 opacity-70",
        ButtonSize::Dense => "h-3.5 w-3.5 shrink-0 opacity-70",
    }
}
