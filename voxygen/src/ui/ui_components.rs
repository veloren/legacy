use ui::UiInternal;

use conrod::{
    widget,
    Widget,
    Positionable,
    Colorable,
    color,
    Borderable,
};

#[derive(Copy, Clone, Debug)]
pub struct UiState {
    pub fps: bool,
    pub version: bool,
    pub chat: bool,
    pub menu: bool,
    pub menupage: MenuPage,
}

impl UiState {
    pub fn normal_game() -> Self {
        Self {
            fps: true,
            version: true,
            chat: false,
            menu: false,
            menupage: MenuPage::Main,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MenuPage {
    Main,
}

pub fn render(ui: &mut UiInternal) {
    let master_id = ui.generate_widget_id();
    let left_col_id = ui.generate_widget_id();
    let mid_col_id = ui.generate_widget_id();
    let right_col_id = ui.generate_widget_id();
    let right_col_top = ui.generate_widget_id();
    let right_col_bot = ui.generate_widget_id();
    let right_col_bot_version = ui.generate_widget_id();
    let right_col_bot_fps = ui.generate_widget_id();
    let version_id = ui.generate_widget_id();
    let fps_id = ui.generate_widget_id();

    let width = ui.get_width();
    let height = ui.get_height();

    let state = ui.get_state();
    let fps = ui.fps.tick();

    let uicell = &mut ui.get_ui_cell();

    let right_col_layout = [
        (right_col_bot_fps, widget::Canvas::new().color(color::TRANSPARENT).border(0.0)),
        (right_col_bot_version, widget::Canvas::new().color(color::TRANSPARENT).border(0.0)),
    ];

    let right_col = [
        (right_col_top, widget::Canvas::new().color(color::TRANSPARENT).length_weight(0.9).border(0.0)),
        (right_col_bot, widget::Canvas::new().color(color::TRANSPARENT).length_weight(0.1).border(0.0).flow_down(&right_col_layout)),
    ];

    let master_cols = [
        (left_col_id, widget::Canvas::new().color(color::TRANSPARENT).length_weight(0.3).border(0.0)),
        (mid_col_id, widget::Canvas::new().color(color::TRANSPARENT).length_weight(0.5).border(0.0)),
        (right_col_id,widget::Canvas::new().color(color::TRANSPARENT).length_weight(0.2).border(0.0).flow_down(&right_col)),
    ];


    widget::Canvas::new().
        flow_right(&master_cols)
        .color(color::TRANSPARENT)
        .border(0.0)
        .set(master_id, uicell);


    if state.version {
        widget::Text::new(&format!("Version {}", env!("CARGO_PKG_VERSION")))
            .color(color::DARK_CHARCOAL)
            .mid_right_with_margin_on(right_col_bot_version, 10.0)
            .right_justify()
            .font_size((height * 0.03) as u32)
            .line_spacing(10.0)
            .set(version_id, uicell);
    }

    if state.fps {
        widget::Text::new(&format!("Fps {}", fps))
            .color(color::DARK_CHARCOAL)
            .mid_right_with_margin_on(right_col_bot_fps, 10.0)
            .right_justify()
            .font_size((height * 0.03) as u32)
            .line_spacing(10.0)
            .set(fps_id, uicell);
    }
}
