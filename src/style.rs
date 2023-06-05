use egui::{Rounding, Color32, Stroke, Ui};

#[derive(Clone)]
pub struct Style {
    pub separator_width: f32,
    pub active_background: Color32,
    pub background: Color32,
    pub selection_color: Color32,
    pub tab_color: Color32,
    pub close_tab_color: Color32,
    pub tab_rounding: Rounding,
    pub active_text_color: Color32,
    pub text_color: Color32,
    pub separator: Color32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            separator_width: 3.0,
            active_background: Color32::from_gray(0x1E),
            background: Color32::from_gray(0x1E),
            selection_color: Color32::from_rgba_unmultiplied(61, 133, 224, 60),
            close_tab_color: Color32::from_gray(0xaa),
            tab_color: Color32::from_gray(0x30),
            tab_rounding: Rounding::same(2.0),
            active_text_color: Color32::from_gray(0xff),
            text_color: Color32::from_gray(0xaa),
            separator: Color32::from_gray(0x28),
        }
    }
}

impl Style {
    pub fn dock(&self) -> egui_dock::Style {
        egui_dock::Style {
            dock_area_padding: None,
            selection_color: self.selection_color,
            border: Stroke {
                color: self.text_color,
                ..Stroke::default()
            },
            buttons: egui_dock::ButtonsStyle {
                close_tab_color: self.close_tab_color,
                close_tab_active_color: self.close_tab_color,
                close_tab_bg_fill: Color32::TRANSPARENT,
                ..Default::default()
            },
            separator: egui_dock::SeparatorStyle {
                width: 3.0,
                color_idle: self.separator,
                color_hovered: self.separator,
                color_dragged: self.separator,
                ..Default::default()
            },
            tab_bar: egui_dock::TabBarStyle {
                bg_fill: self.separator,
                height: 30.0,
                show_scroll_bar_on_overflow: false,
                rounding: self.tab_rounding,
                hline_color: self.separator,
            },
            tabs: egui_dock::TabStyle {
                bg_fill: self.tab_color,
                text_color_focused: self.active_text_color,
                text_color_unfocused: self.text_color,
                text_color_active_focused: self.active_text_color,
                text_color_active_unfocused: self.text_color,
                ..Default::default()
            },
        }
    }

    pub fn set_theme_visuals(&self, ui: &mut Ui) {
        let visuals = ui.visuals_mut();

        let expansion = 0.0;
        visuals.widgets.noninteractive.expansion = expansion;
        visuals.widgets.inactive.expansion = expansion;
        visuals.widgets.hovered.expansion = expansion;
        visuals.widgets.active.expansion = expansion;
        visuals.widgets.open.expansion = expansion;

        visuals.widgets.noninteractive.bg_fill = self.background;
        visuals.widgets.inactive.bg_fill = self.background;
        visuals.widgets.hovered.bg_fill = self.background;
        visuals.widgets.active.bg_fill = self.active_background;
        visuals.widgets.open.bg_fill = self.active_background;

        visuals.widgets.noninteractive.fg_stroke.color = self.text_color;
        visuals.widgets.inactive.fg_stroke.color = self.text_color;
        visuals.widgets.hovered.fg_stroke.color = self.text_color;
        visuals.widgets.active.fg_stroke.color = self.active_text_color;
        visuals.widgets.open.fg_stroke.color = self.active_text_color;

        visuals.extreme_bg_color = self.active_background;
        visuals.widgets.noninteractive.rounding = self.tab_rounding;
        visuals.widgets.inactive.rounding = self.tab_rounding;
        visuals.widgets.hovered.rounding = self.tab_rounding;
        visuals.widgets.active.rounding = self.tab_rounding;
        visuals.widgets.open.rounding = self.tab_rounding;
        visuals.menu_rounding = self.tab_rounding;

        visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(61, 133, 224, 60);
    }

    pub fn for_scrollbar(&self, ui: &mut Ui) {
        let spacing = ui.spacing_mut();
        spacing.scroll_bar_width = 4.0;

        let visuals = ui.visuals_mut();

        visuals.extreme_bg_color = Color32::from_gray(0x2B);
        visuals.clip_rect_margin = 0.0;
        visuals.widgets.noninteractive.rounding = self.tab_rounding;
        visuals.widgets.inactive.rounding = self.tab_rounding;
        visuals.widgets.hovered.rounding = self.tab_rounding;
        visuals.widgets.active.rounding = self.tab_rounding;
        visuals.widgets.open.rounding = self.tab_rounding;
    }

    pub fn scrollarea(&self, ui: &mut Ui) {
        let visuals = ui.visuals_mut();

        visuals.extreme_bg_color = self.active_background;
        visuals.widgets.noninteractive.rounding = self.tab_rounding;
        visuals.widgets.inactive.rounding = self.tab_rounding;
        visuals.widgets.hovered.rounding = self.tab_rounding;
        visuals.widgets.active.rounding = self.tab_rounding;
        visuals.widgets.open.rounding = self.tab_rounding;
    }
}
