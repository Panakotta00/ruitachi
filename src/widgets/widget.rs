use crate::events::{MouseButtonEvent, PointerEvent};

pub struct HoverState {
    pub is_hovered : bool
}

impl Default for HoverState {
    fn default() -> Self {
        HoverState {
            is_hovered: false,
        }
    }
}

pub trait Hoverable {
    fn get_hover_state(&mut self) -> &mut HoverState;

    fn on_mouse_enter(&mut self, event : PointerEvent) {}
    fn on_mouse_leave(&mut self, event : PointerEvent) {}
}

pub trait MouseInteract {
    fn on_mouse_button_down(&mut self, event : MouseButtonEvent) {}
    fn on_mouse_button_up(&mut self, event : MouseButtonEvent) {}
}

pub struct ClickState {
    clicked : bool,
}

impl Default for ClickState {
    fn default() -> Self {
        ClickState {
            clicked: false,
        }
    }
}

impl ClickState {
    fn on_mouse_button_down(&mut self, event : MouseButtonEvent) {
        self.clicked = true
    }

    fn on_mouse_button_up(&mut self, clickable : &mut dyn Clickable, event : MouseButtonEvent) {
        if self.clicked {
            clickable.on_click(event);
        }
        self.clicked = false;
    }

    fn on_mouse_move(&mut self, event : PointerEvent) {

    }
}

pub trait Clickable : MouseInteract + Hoverable {
    fn get_click_state(&mut self) -> &mut ClickState;

    fn on_click(&mut self, event : MouseButtonEvent) {}
}

pub trait Widget {
    fn get_parent(&self) -> Option<&mut Self>;

    fn paint(painter : &mut crate::paint::Painter);

    fn on_click(event : MouseButtonEvent);
    fn on_hover(event : PointerEvent);
}

pub struct Button {
    text : String,

    hover_state : HoverState,
}

impl Hoverable for Button {
    fn get_hover_state(&mut self) -> &mut HoverState {
        &mut self.hover_state
    }
}