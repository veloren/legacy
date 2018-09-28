// Standard
use std::rc::Rc;

// Library
use vek::*;

// Local
use new_ui::{
    element::{Button, HBox, Label, Rect, VBox, WinBox},
    Span, Ui,
};
use renderer::Renderer;
use window::Event;

pub struct Hud {
    ui: Ui,
    debug_box: DebugBox,
    chat_box: ChatBox,
}

impl Hud {
    pub fn new() -> Hud {
        let winbox = WinBox::new();

        let hotbar = HBox::new()
            .with_color(Rgba::new(0.0, 0.0, 0.0, 0.5))
            .with_margin(Span::px(8, 8));
        for _ in 0..5 {
            hotbar.push_back(
                Rect::new()
                    .with_color(Rgba::new(1.0, 0.8, 0.3, 1.0))
                    .with_padding(Span::px(8, 8)),
            );
        }
        winbox.add_child_at(
            Span::bottom(),
            Span::bottom() + Span::px(0, 16),
            Span::px(296, 72),
            hotbar,
        );

        let debug_box = DebugBox::new();
        winbox.add_child_at(
            Span::top_left(),
            Span::top_left() + Span::px(-16, -16),
            Span::px(366, 112),
            debug_box.root(),
        );

        let chat_box = ChatBox::new();
        winbox.add_child_at(
            Span::bottom_left(),
            Span::bottom_left() + Span::px(-16, 16),
            Span::px(316, 176),
            chat_box.root(),
        );

        winbox.add_child_at(
            Span::top_right(),
            Span::top_right() + Span::px(16, -16),
            Span::px(128, 64),
            Button::new()
                .with_color(Rgba::new(1.0, 0.0, 0.0, 1.0))
                .with_hover_color(Rgba::new(0.0, 1.0, 0.0, 1.0))
                .with_click_color(Rgba::new(0.0, 0.0, 1.0, 1.0))
                .with_click_fn(|_| println!("Clicked the button!"))
                .with_margin(Span::px(8, 8))
                .with_child(Label::new().with_color(Rgba::one()).with_text("Click me!".to_string())),
        );

        Hud {
            ui: Ui::new(winbox),
            debug_box,
            chat_box,
        }
    }

    pub fn debug_box(&self) -> &DebugBox { &self.debug_box }
    pub fn chat_box(&self) -> &ChatBox { &self.chat_box }

    pub fn render(&mut self, renderer: &mut Renderer) { self.ui.render(renderer); }
    pub fn handle_event(&self, event: &Event, renderer: &mut Renderer) -> bool { self.ui.handle_event(event, renderer) }
}

pub struct DebugBox {
    pub version_label: Rc<Label>,
    pub githash_label: Rc<Label>,
    pub buildtime_label: Rc<Label>,
    pub fps_label: Rc<Label>,
    pub pos_label: Rc<Label>,
    vbox: Rc<VBox>,
}

impl DebugBox {
    fn new() -> Self {
        let vbox = VBox::new()
            .with_color(Rgba::new(0.0, 0.0, 0.0, 0.5))
            .with_margin(Span::px(8, 8));

        vbox.push_back(
            Label::new()
                .with_text("Debug".to_string())
                .with_size(Span::px(16, 16))
                .with_color(Rgba::new(1.0, 1.0, 1.0, 1.0)),
        );

        let template_label = Label::new()
            .with_size(Span::px(16, 16))
            .with_color(Rgba::new(1.0, 1.0, 1.0, 0.7));

        let version_label = vbox.push_back(template_label.clone_all());
        let githash_label = vbox.push_back(template_label.clone_all());
        let buildtime_label = vbox.push_back(template_label.clone_all());
        let fps_label = vbox.push_back(template_label.clone_all());
        let pos_label = vbox.push_back(template_label.clone_all());

        Self {
            version_label,
            githash_label,
            buildtime_label,
            fps_label,
            pos_label,
            vbox,
        }
    }

    fn root(&self) -> Rc<VBox> { self.vbox.clone() }
}

pub struct ChatBox {
    vbox: Rc<VBox>,
    template_label: Rc<Label>,
}

impl ChatBox {
    fn new() -> Self {
        let max_msgs = 10;

        let vbox = VBox::new()
            .with_color(Rgba::new(0.0, 0.0, 0.0, 0.5))
            .with_margin(Span::px(8, 8));

        let template_label = Label::new()
            .with_size(Span::px(16, 16))
            .with_color(Rgba::new(1.0, 1.0, 1.0, 0.7));

        for _ in 0..max_msgs {
            vbox.push_back(template_label.clone_all());
        }

        Self { vbox, template_label }
    }

    pub fn add_chat_msg(&self, text: String) {
        self.vbox.pop_front();
        self.vbox.push_back(self.template_label.clone_all().with_text(text));
    }

    fn root(&self) -> Rc<VBox> { self.vbox.clone() }
}
