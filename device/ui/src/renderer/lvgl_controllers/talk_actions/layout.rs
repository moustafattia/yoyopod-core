use super::*;

impl TalkActionsController {
    pub(super) fn sync_primary_layout(
        &self,
        facade: &mut dyn LvglFacade,
        model: &TalkActionsViewModel,
        action_count: usize,
    ) -> Result<()> {
        if let Some(title_label) = self.title_label {
            facade.set_visible(title_label, false)?;
        }
        for dot in &self.dots {
            facade.set_visible(*dot, false)?;
        }

        for index in 0..self.buttons.len() {
            let visible = index == 0 && action_count > 0;
            facade.set_visible(self.buttons[index], visible)?;
            if !visible {
                continue;
            }
            let color = color_for_kind(model.buttons[0].color_kind);
            facade.set_geometry(self.buttons[index], 76, 126, 88, 88)?;
            facade.set_variant(self.buttons[index], "talk_action_primary", color)?;
            facade.set_geometry(self.button_labels[index], 24, 32, 40, 24)?;
            facade.set_icon(self.button_labels[index], &model.buttons[0].icon_key)?;
            facade.set_variant(self.button_labels[index], "talk_action_primary", color)?;
        }

        if let Some(status_label) = self.status_label {
            facade.set_geometry(status_label, 30, 220, 180, 16)?;
            facade.set_visible(status_label, true)?;
            facade.set_text(status_label, &model.status)?;
            facade.set_variant(
                status_label,
                "talk_action_status",
                color_for_kind(model.status_kind),
            )?;
        }
        Ok(())
    }

    pub(super) fn sync_action_row(
        &self,
        facade: &mut dyn LvglFacade,
        model: &TalkActionsViewModel,
        action_count: usize,
        selected_index: usize,
    ) -> Result<()> {
        if let Some(status_label) = self.status_label {
            facade.set_visible(status_label, false)?;
        }

        let diameter = if model.button_size_kind == 1 { 64 } else { 56 };
        let gap = if model.button_size_kind == 1 { 16 } else { 12 };
        let center_y = if model.button_size_kind == 1 {
            154
        } else {
            156
        };
        let row_width =
            (action_count as i32 * diameter) + (action_count.saturating_sub(1) as i32 * gap);
        let start_x = 120 - (row_width / 2);
        let title_y = center_y + (diameter / 2) + 16;

        for index in 0..self.buttons.len() {
            if index >= action_count {
                facade.set_visible(self.buttons[index], false)?;
                continue;
            }
            let selected = index == selected_index;
            let color = color_for_kind(model.buttons[index].color_kind);
            facade.set_visible(self.buttons[index], true)?;
            facade.set_geometry(
                self.buttons[index],
                start_x + (index as i32 * (diameter + gap)),
                center_y - (diameter / 2),
                diameter,
                diameter,
            )?;
            facade.set_variant(
                self.buttons[index],
                if selected {
                    "talk_action_selected"
                } else {
                    "talk_action_unselected"
                },
                color,
            )?;
            let label_xy = (diameter - 40) / 2;
            facade.set_geometry(self.button_labels[index], label_xy, label_xy + 1, 40, 24)?;
            facade.set_icon(self.button_labels[index], &model.buttons[index].icon_key)?;
            facade.set_variant(
                self.button_labels[index],
                if selected {
                    "talk_action_selected"
                } else {
                    "talk_action_unselected"
                },
                color,
            )?;
        }

        if let Some(title_label) = self.title_label {
            facade.set_geometry(title_label, 30, title_y, 180, 22)?;
            facade.set_visible(title_label, true)?;
            facade.set_text(title_label, &model.title)?;
        }

        for (index, dot) in self.dots.iter().copied().enumerate() {
            if index >= action_count {
                facade.set_visible(dot, false)?;
                continue;
            }
            let selected = index == selected_index;
            let size = if selected { 8 } else { 6 };
            facade.set_visible(dot, true)?;
            facade.set_geometry(
                dot,
                120 - ((action_count as i32 * 14) / 2) + (index as i32 * 14),
                title_y + 30,
                size,
                size,
            )?;
            facade.set_selected(dot, selected)?;
            facade.set_accent(
                dot,
                if selected {
                    ACCENT
                } else {
                    mix_u24(ACCENT, BACKGROUND, 68)
                },
            )?;
        }
        Ok(())
    }
}
fn color_for_kind(kind: i32) -> u32 {
    match kind {
        1 => SUCCESS,
        2 => WARNING,
        3 => ERROR,
        4 => NEUTRAL,
        _ => ACCENT,
    }
}
fn mix_u24(primary_rgb: u32, secondary_rgb: u32, secondary_ratio_percent: u8) -> u32 {
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
