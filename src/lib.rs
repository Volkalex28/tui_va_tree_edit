pub use argument::Args;
pub use argument::NumberType;
pub use argument::StringType;
pub use argument::Value;
pub use argument::Type;
use linked_hash_map::LinkedHashMap;
pub use state::Node;
pub use tree::Branch;
pub use tree::Tree;
use tui::style::Style;

mod argument;
pub mod state;
mod tree;
mod widget;

pub type Branches<'a> = LinkedHashMap<String, Branch<'a>>;
type BranchItem<'a, 'b> = (&'b String, &'b Branch<'a>);
type BranchItemMut<'a, 'b> = (&'b String, &'b mut Branch<'a>);

#[derive(Debug, PartialEq)]
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
    Char(char),
}

#[derive(Default, Clone)]
pub struct TreeEdit<'a> {
    title: String,
    tabs: Branches<'a>,
    state: state::State,
}
impl<'a> TreeEdit<'a> {
    pub fn new<T: Into<String>>(title: T) -> Self {
        Self {
            title: title.into(),
            tabs: Default::default(),
            state: Default::default(),
        }
    }

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
        self.tabs
            .iter()
            .skip(self.get_index_tab().unwrap_or(0))
            .next()
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
