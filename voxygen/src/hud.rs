// Standard
use std::rc::Rc;

// Library
use vek::*;

// Local
use new_ui::{
    Ui,
    ResCache,
    Span,
    element::{
        Element,
        WinBox,
        HBox,
        VBox,
        Label,
        Rect,
    },
};
use renderer::Renderer;

pub struct Hud {
    ui: Ui,
    rescache: ResCache,
    debug_box: DebugBox,
    chat_box: ChatBox,
}

impl Hud {
    pub fn new() -> Hud {
        let mut winbox = WinBox::new();

        let mut hotbar = HBox::new()
            .with_color(Rgba::new(0.0, 0.0, 0.0, 0.5))
            .with_margin(Span::px(8, 8));
        for _ in 0..5 {
            hotbar.push_back(Rect::new()
                .with_color(Rgba::new(1.0, 0.8, 0.3, 1.0))
                .with_padding(Span::px(8, 8))
            );
        }
        winbox.add_child_at(Span::bottom(), Span::bottom() + Span::px(0, 16), Span::px(296, 72), hotbar);

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

        Hud {
            ui: Ui::new(winbox),
            rescache: ResCache::new(),
            debug_box,
            chat_box,
        }
    }

    pub fn debug_box(&self) -> &DebugBox { &self.debug_box }
    pub fn chat_box(&self) -> &ChatBox { &self.chat_box }

    pub fn render(&mut self, renderer: &mut Renderer) {
        self.ui.render(renderer, &mut self.rescache);
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
        let mut vbox = VBox::new()
            .with_color(Rgba::new(0.0, 0.0, 0.0, 0.5))
            .with_margin(Span::px(8, 8));

        vbox.push_back(Label::new()
            .with_text("Debug".to_string())
            .with_size(Span::px(16, 16))
            .with_color(Rgba::new(1.0, 1.0, 1.0, 1.0))
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

    fn root(&self) -> Rc<VBox> {
        self.vbox.clone()
    }
}

pub struct ChatBox {
    max_msgs: usize,
    vbox: Rc<VBox>,
    template_label: Rc<Label>,
}

impl ChatBox {
    fn new() -> Self {
        let max_msgs = 10;

        let mut vbox = VBox::new()
            .with_color(Rgba::new(0.0, 0.0, 0.0, 0.5))
            .with_margin(Span::px(8, 8));

        let template_label = Label::new()
            .with_size(Span::px(16, 16))
            .with_color(Rgba::new(1.0, 1.0, 1.0, 0.7));

        for _ in 0..max_msgs {
            vbox.push_back(template_label.clone_all());
        }

        Self {
            max_msgs,
            vbox,
            template_label,
        }
    }

    pub fn add_chat_msg(&self, text: String) {
        self.vbox.pop_front();
        self.vbox.push_back(self.template_label
            .clone_all()
            .with_text(text)
        );
    }

    fn root(&self) -> Rc<VBox> {
        self.vbox.clone()
    }
}
