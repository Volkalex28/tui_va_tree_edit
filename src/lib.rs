pub use argument::{
    value::{NumberType, StringType, Type, Value},
    Args,
};
pub use array::Array;
pub use branch::{Branch, Branches};
pub use state::Node;
pub use tree::Tree;

use tui::style::Style;

mod argument;
mod array;
mod branch;
pub mod state;
mod tree;
mod widget;

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    NextTab,
    PreviousTab,
    NextItem,
    PreviousItem,
    NextLevel,
    PreviousLevel,
    Enter,
    Cancel,
    Backspace,
    Delete,
    Char(char),
}

#[derive(Default, Clone)]
pub struct TreeEdit<'a> {
    title: String,
    tabs: Branches<'a>,
    state: state::State,
}
impl TreeEdit<'_> {
    pub fn new<T: ToString>(title: T) -> Self {
        Self {
            title: title.to_string(),
            tabs: Default::default(),
            state: Default::default(),
        }
    }
}

impl<'a> TreeEdit<'a> {
    pub fn tab(mut self, tab_name: String, tab: impl Into<Branch<'a>>) -> Self {
        self.tabs.insert(tab_name, tab.into());
        if self.get_index_tab().is_none() {
            self.transition(Event::PreviousTab)
        }
        self
    }

    pub fn get_tabs(&self) -> &Branches<'a> {
        &self.tabs
    }

    pub fn widget<'b>(&'b self) -> widget::Drawer<'a, 'b> {
        widget::Drawer::new(self)
    }

    fn get_index_tab(&self) -> Option<usize> {
        self.state.index_tab(&self.tabs)
    }
    pub fn get_current_tab(&self) -> Option<(&String, &Branch<'a>)> {
        self.tabs.iter().nth(self.get_index_tab().unwrap_or(0))
    }

    pub fn transition(&mut self, event: Event) {
        self.state.transition(event, &mut self.tabs)
    }

    pub fn position(&self) -> &Vec<Node> {
        &self.state.position
    }

    pub fn style(mut self, style: Style) -> Self {
        self.state.style = style;
        self
    }
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.state.highlight_style = style;
        self
    }

    pub fn in_input_mode(&self) -> bool {
        self.state.input.is_some()
    }
}
