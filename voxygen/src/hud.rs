// Standard
use std::{
    cell::RefCell,
    mem,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

// Library
use vek::*;

// Local
use renderer::Renderer;
use ui::{
    element::{HBox, Label, Rect, TextBox, VBox, WinBox},
    Span, Ui,
};
use window::Event;

pub enum HudEvent {
    ChatMsgSent { text: String },
}

pub struct Hud {
    ui: Ui,
    debug_box: DebugBox,
    chat_box: ChatBox,
    chatbox_input: Rc<TextBox>,

    chat_enabled: Rc<AtomicBool>,
    events: Rc<RefCell<Vec<HudEvent>>>,
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
            Span::bottom_left() + Span::px(-16, 56),
            Span::px(316, 176),
            chat_box.root(),
        );

        let chat_enabled = Rc::new(AtomicBool::new(false));
        let events = Rc::new(RefCell::new(vec![]));

        let chat_enabled_ref = chat_enabled.clone();
        let events_ref = events.clone();

        let chatbox_input = TextBox::new()
            .with_color(Rgba::new(1.0, 1.0, 1.0, 1.0))
            .with_background_color(Rgba::new(0.0, 0.0, 0.0, 0.8))
            .with_margin(Span::px(8, 8))
            .with_return_fn(move |chatbox_input, text| {
                if chat_enabled_ref.load(Ordering::Relaxed) {
                    events_ref
                        .borrow_mut()
                        .push(HudEvent::ChatMsgSent { text: text.to_string() });
                    chat_enabled_ref.store(false, Ordering::Relaxed);
                }
                chatbox_input.set_background_color(Rgba::new(0.0, 0.0, 0.0, 0.8));
            })
            .with_text("".to_string());

        winbox.add_child_at(
            Span::bottom_left(),
            Span::bottom_left() + Span::px(-16, 16),
            Span::px(316, 32),
            chatbox_input.clone(),
        );

        Hud {
            ui: Ui::new(winbox),
            debug_box,
            chat_box,
            chatbox_input,

            chat_enabled,
            events,
        }
    }

    pub fn debug_box(&self) -> &DebugBox { &self.debug_box }
    pub fn chat_box(&self) -> &ChatBox { &self.chat_box }

    pub fn get_events(&self) -> Vec<HudEvent> {
        let mut events = vec![];
        mem::swap(&mut *self.events.borrow_mut(), &mut events);
        events
    }

    pub fn render(&mut self, renderer: &mut Renderer) { self.ui.render(renderer); }
    pub fn handle_event(&self, event: &Event, renderer: &mut Renderer) -> bool {
        match event {
            Event::Character { ch } => {
                if self.chat_enabled.load(Ordering::Relaxed) {
                    self.ui.handle_event(event, renderer)
                } else {
                    if *ch == '\n' || *ch == '\r' {
                        self.chat_enabled.store(true, Ordering::Relaxed);
                        self.chatbox_input.set_background_color(Rgba::new(0.0, 0.0, 0.3, 0.8));

                        true
                    } else {
                        false
                    }
                }
            },
            Event::KeyboardInput { i, device } => {
                if self.chat_enabled.load(Ordering::Relaxed) {
                    self.ui.handle_event(event, renderer)
                } else {
                    false
                }
            },
            _ => self.ui.handle_event(event, renderer),
        }
    }
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
