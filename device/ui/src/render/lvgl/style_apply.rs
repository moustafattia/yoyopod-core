mod accent;
mod base;
mod icons;
mod tuning;
mod variant;

pub(crate) use accent::apply_accent_raw;
pub(crate) use base::{apply_style_raw, hide_widget_raw, reset_style_raw};
pub(crate) use icons::icon_label;
pub(crate) use tuning::apply_role_tuning_raw;
pub(crate) use variant::apply_variant_raw;

pub(crate) fn mix_u24(primary_rgb: u32, secondary_rgb: u32, secondary_ratio_percent: u8) -> u32 {
    let secondary_ratio = u32::from(secondary_ratio_percent.min(100));
    let primary_ratio = 100 - secondary_ratio;
    let red = ((((primary_rgb >> 16) & 0xFF) * primary_ratio
        + ((secondary_rgb >> 16) & 0xFF) * secondary_ratio)
        / 100)
        & 0xFF;
    let green = ((((primary_rgb >> 8) & 0xFF) * primary_ratio
        + ((secondary_rgb >> 8) & 0xFF) * secondary_ratio)
        / 100)
        & 0xFF;
    let blue = (((primary_rgb & 0xFF) * primary_ratio + (secondary_rgb & 0xFF) * secondary_ratio)
        / 100)
        & 0xFF;
    (red << 16) | (green << 8) | blue
}
