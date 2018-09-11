// Standard
use std::{cmp::min, collections::VecDeque};

// Library
use conrod::{self, color, widget, Borderable, Colorable, Positionable, Sizeable, Widget};

// Local
use get_build_time;
use get_git_hash;
use ui::{Ui, UiInternalEvent};

pub const MAX_CHAT_LINES: usize = 12;

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
            show_chat: false,
            show_menu: false,
            menupage: MenuPage::Main,
            chat_lines: VecDeque::with_capacity(MAX_CHAT_LINES),
            chat_message: String::new(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MenuPage {
    Main,
}

pub fn render(ui: &mut Ui) {
    let master_id = ui.get_widget_id("master_id");
    let top_id = ui.get_widget_id("top_id");
    let bottom_id = ui.get_widget_id("bottom_id");
    let top_left_id = ui.get_widget_id("top_left_id");
    let top_mid_id = ui.get_widget_id("top_mid_id");
    let top_right_id = ui.get_widget_id("top_right_id");
    let top_right_top_id = ui.get_widget_id("top_right_top_id");
    let top_right_bot1_id = ui.get_widget_id("top_right_bot1_id");
    let top_right_bot2_id = ui.get_widget_id("top_right_bot2_id");
    let top_right_bot3_id = ui.get_widget_id("top_right_bot3_id");
    let top_right_bot4_id = ui.get_widget_id("top_right_bot4_id");

    let chat_lines = ui.get_widget_id("chat_lines");
    let chat_background = ui.get_widget_id("chat_background");

    let version_id = ui.get_widget_id("version_id");
    let fps_id = ui.get_widget_id("fps_id");
    let text_id = ui.get_widget_id("text_id");
    let build_hash_id = ui.get_widget_id("build_hash_id");
    let build_time_id = ui.get_widget_id("build_time_id");

    let width = ui.get_width();
    let height = ui.get_height();

    let state = ui.get_state();
    let fps = ui.fps.tick();

    let event_tx = ui.get_event_tx();

    let event_focus = conrod::event::Event::Ui(conrod::event::Ui::WidgetCapturesInputSource(
        text_id,
        conrod::input::Source::Keyboard,
    ));

    let tx = event_tx.clone();
    ui.widget_events(text_id, |event| match event {
        conrod::event::Widget::Press(press) => match press.button {
            conrod::event::Button::Keyboard(key) => match key {
                conrod::input::Key::Return => {
                    tx.send(UiInternalEvent::SendChat).unwrap();
                },
                _ => (),
            },
            _ => (),
        },
        _ => (),
    });

    let uicell = &mut ui.get_ui_cell();

    let splits_right = [
        (
            top_right_top_id,
            widget::Canvas::new()
                .color(color::TRANSPARENT)
                .border(0.0)
                .length_weight(0.9),
        ),
        (
            top_right_bot1_id,
            widget::Canvas::new()
                .color(color::TRANSPARENT)
                .border(0.0)
                .length_weight(0.05),
        ),
        (
            top_right_bot2_id,
            widget::Canvas::new()
                .color(color::TRANSPARENT)
                .border(0.0)
                .length_weight(0.05),
        ),
        (
            top_right_bot3_id,
            widget::Canvas::new()
                .color(color::TRANSPARENT)
                .border(0.0)
                .length_weight(0.05),
        ),
        (
            top_right_bot4_id,
            widget::Canvas::new()
                .color(color::TRANSPARENT)
                .border(0.0)
                .length_weight(0.05),
        ),
    ];
    let top_splits = [
        (
            top_left_id,
            widget::Canvas::new()
                .color(color::TRANSPARENT)
                .border(0.0)
                .length_weight(1.0 / 3.0),
        ),
        (
            top_mid_id,
            widget::Canvas::new()
                .color(color::TRANSPARENT)
                .border(0.0)
                .length_weight(1.0 / 3.0),
        ),
        (
            top_right_id,
            widget::Canvas::new()
                .color(color::TRANSPARENT)
                .border(0.0)
                .length_weight(1.0 / 3.0)
                .flow_down(&splits_right),
        ),
    ];

    let master_splits = [
        (
            top_id,
            widget::Canvas::new()
                .color(color::TRANSPARENT)
                .border(0.0)
                .length_weight(0.96)
                .flow_right(&top_splits),
        ),
        (
            bottom_id,
            widget::Canvas::new()
                .color(color::TRANSPARENT)
                .border(0.0)
                .length_weight(0.04),
        ),
    ];

    widget::Canvas::new()
        .flow_down(&master_splits)
        .color(color::TRANSPARENT)
        .border(0.0)
        .scroll_kids_horizontally()
        .set(master_id, uicell);

    if state.chat_lines.len() != 0 {
        let (mut items, scrollbar) = widget::List::flow_down(state.chat_lines.len())
            .item_size(20.0)
            .scrollbar_on_top()
            .top_left_of(master_id)
            .wh_of(master_id)
            .set(chat_lines, uicell);

        while let Some(item) = items.next(uicell) {
            let i = item.i;
            let (alias, msg) = &state.chat_lines[i];
            let label = format!("{}: {}", alias, msg);

            let text = widget::Text::new(&label)
                .color(color::BLACK)
                .font_size(16)
                .left_justify();

            item.set(text, uicell);
        }

        if let Some(s) = scrollbar {
            s.set(uicell)
        }
    }

    if state.show_fps {
        widget::Text::new(&format!("Fps: {}", fps))
            .color(color::BLACK)
            .font_size(((0.01 + height) * 0.03) as u32)
            .right_justify()
            .mid_right_with_margin_on(top_right_bot3_id, 5.0)
            .set(fps_id, uicell);
    }

    if state.show_version {
        widget::Text::new(&format!("Version {}", env!("CARGO_PKG_VERSION")))
            .color(color::BLACK)
            .font_size((height * 0.03) as u32)
            .right_justify()
            .mid_right_with_margin_on(top_right_bot4_id, 5.0)
            .set(version_id, uicell);
    }

    if state.show_chat {
        uicell.global_input_mut().current.widget_capturing_keyboard = Some(text_id);

        widget::Canvas::new()
            .bottom_left_of(bottom_id)
            .wh_of(bottom_id)
            .color(color::CHARCOAL)
            .set(chat_background, uicell);

        for edit in widget::TextEdit::new(&state.chat_message)
            .color(color::WHITE)
            .wh_of(chat_background)
            .bottom_left_with_margins_on(chat_background, 0.0, 5.0)
            .left_justify()
            .align_text_y_middle()
            .font_size((height * 0.03) as u32)
            .restrict_to_height(true)
            .set(text_id, uicell)
        {
            event_tx.send(UiInternalEvent::UpdateChatText(edit)).unwrap();
        }
    }

    let git_hash = get_git_hash();
    widget::Text::new(&format!("Build {}", &git_hash[..min(8, git_hash.len())]))
        .color(color::BLACK)
        .font_size((height * 0.03) as u32)
        .right_justify()
        .mid_right_with_margin_on(top_right_bot1_id, 5.0)
        .set(build_hash_id, uicell);

    widget::Text::new(&format!("Built at {}", get_build_time()))
        .color(color::BLACK)
        .font_size((height * 0.03) as u32)
        .right_justify()
        .mid_right_with_margin_on(top_right_bot2_id, 5.0)
        .set(build_time_id, uicell);
}
