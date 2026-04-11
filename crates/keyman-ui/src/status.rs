use gpui::*;
use gpui_component::{ActiveTheme, h_flex};

/// Status indicator shown in the UI.
#[derive(Clone, Copy, PartialEq)]
pub enum AppStatus {
    Enabled,
    InGame,
    Paused,
}

impl AppStatus {
    pub fn label(&self) -> String {
        match self {
            Self::Enabled => keyman_core::i18n::t("enabled").to_string(),
            Self::InGame => keyman_core::i18n::t("in_game").to_string(),
            Self::Paused => keyman_core::i18n::t("paused").to_string(),
        }
    }

    pub fn color(&self, _cx: &App) -> Hsla {
        match self {
            Self::Enabled => gpui::green(),
            Self::InGame => gpui::blue(),
            Self::Paused => gpui::yellow(),
        }
    }
}

pub fn status_indicator(status: AppStatus, cx: &App) -> impl IntoElement {
    let color = status.color(cx);

    h_flex()
        .gap_1()
        .items_center()
        .child(
            div()
                .size(px(8.))
                .rounded_full()
                .bg(color)
        )
        .child(
            div().text_xs().child(status.label())
        )
        .child(
            div().text_xs().text_color(cx.theme().muted_foreground).child(keyman_core::i18n::t("f12_toggle"))
        )
}
