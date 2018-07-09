use ui::{
    Ui,
    UiInternalEvent,
};

use std::collections::VecDeque;
pub const MAX_CHAT_LINES: usize = 8;

use conrod::{
    widget,
    Widget,
    Positionable,
    Colorable,
    color,
    Borderable,
    Sizeable,
};

#[derive(Clone, Debug)]
pub struct UiState {
    pub show_fps: bool,
    pub show_version: bool,
    pub show_chat: bool,
    pub show_menu: bool,
    pub menupage: MenuPage,
    pub chat_lines: VecDeque<(String, String)>,
    pub chat_message: String,
}

impl UiState {
    pub fn normal_game() -> Self {
        Self {
            show_fps: true,
            show_version: true,
            show_chat: true,
            show_menu: false,
            menupage: MenuPage::Main,
            chat_lines: VecDeque::with_capacity(MAX_CHAT_LINES),
            chat_message: "ABC".to_string(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MenuPage {
    Main,
}

pub fn render(ui: &mut Ui) {
    let master_id = ui.get_widget_id("master_id");
    let left_col_id = ui.get_widget_id("left_col_id");
    let mid_col_id = ui.get_widget_id("mid_col_id");
    let right_col_id = ui.get_widget_id("right_col_id");
    let right_col_top = ui.get_widget_id("right_col_top");
    let right_col_bot = ui.get_widget_id("right_col_bot");
    let right_col_bot_version = ui.get_widget_id("right_col_bot_version");
    let right_col_bot_fps = ui.get_widget_id("right_col_bot_fps");
    let version_id = ui.get_widget_id("version_id");
    let fps_id = ui.get_widget_id("fps_id");
    let text_id = ui.get_widget_id("text_id");

    let width = ui.get_width();
    let height = ui.get_height();

    let state = ui.get_state();
    let fps = ui.fps.tick();

    let event_tx = ui.get_event_tx();

    let uicell = &mut ui.get_ui_cell();

    let right_col_layout = [
        (right_col_bot_fps, widget::Canvas::new().color(color::TRANSPARENT).border(1.0)),
        (right_col_bot_version, widget::Canvas::new().color(color::TRANSPARENT).border(1.0)),
    ];

    let right_col = [
        (right_col_top, widget::Canvas::new().color(color::TRANSPARENT).length_weight(0.9).border(1.0)),
        (right_col_bot, widget::Canvas::new().color(color::TRANSPARENT).length_weight(0.1).border(1.0).flow_down(&right_col_layout)),
    ];

    let master_cols = [
        (left_col_id, widget::Canvas::new().color(color::TRANSPARENT).length_weight(0.3).border(1.0)),
        (mid_col_id, widget::Canvas::new().color(color::TRANSPARENT).length_weight(0.5).border(1.0)),
        (right_col_id,widget::Canvas::new().color(color::TRANSPARENT).length_weight(0.2).border(1.0).flow_down(&right_col)),
    ];

    widget::Canvas::new().
        flow_right(&master_cols)
        .color(color::TRANSPARENT)
        .border(1.0)
        .scroll_kids_horizontally()
        .set(master_id, uicell);

    if state.show_version {
        widget::Text::new(&format!("Version {}", env!("CARGO_PKG_VERSION")))
            .color(color::DARK_CHARCOAL)
            .mid_right_with_margin_on(right_col_bot_version, 10.0)
            .right_justify()
            .font_size((height * 0.03) as u32)
            .line_spacing(10.0)
            .set(version_id, uicell);
    }

    if state.show_fps {
        widget::Text::new(&format!("Fps {}", fps))
            .color(color::DARK_CHARCOAL)
            .mid_right_with_margin_on(right_col_bot_fps, 10.0)
            .right_justify()
            .font_size((height * 0.03) as u32)
            .line_spacing(10.0)
            .set(fps_id, uicell);
    }

    if state.show_chat {
        for event in widget::TextBox::new(&state.chat_message)
            .color(color::DARK_CHARCOAL)
            .w_of(master_id)
            .h(50.0)
            .bottom_left_of(master_id)
            .left_justify()
            .set(text_id, uicell) {

            match event {
                widget::text_box::Event::Enter => event_tx.send(UiInternalEvent::SendChat).unwrap(),
                widget::text_box::Event::Update(string) => event_tx.send(UiInternalEvent::UpdateChatText(string)).unwrap(),
            };
        }
    }
}
