pub use argument::Args;
pub use argument::NumberType;
pub use argument::StringType;
pub use argument::Value;
pub use tree::Tree;
pub use tree::Branch;
pub use state::Node;
use tui::style::Style;

mod argument;
pub mod state;
mod tree;
mod widget;

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
    Char(char)
}

#[derive(Default, Clone)]
pub struct TreeEdit<'a> {
    title: String,
    tabs: Vec<Branch<'a>>,
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

    pub fn tab(mut self, tab: impl Into<Branch<'a>>) -> Self {
        self.tabs.push(tab.into());
        if self.get_index_tab().is_none() {
            self.transition(Event::PreviousTab)
        }
        self
    }

    pub fn get_tabs(&self) -> &Vec<Branch<'a>> {
        &self.tabs
    }

    pub fn widget(&'a self) -> widget::Drawer<'a> {
        widget::Drawer::new(self)
    }

    pub fn get_index_tab(&self) -> Option<usize> {
        self.state.index_tab(&self.tabs)
    }
    pub fn get_tab(&self) -> Option<&Branch<'a>> {
        self.tabs.get(self.get_index_tab().unwrap_or(0))
    }

    pub fn transition<'b>(&'b mut self, event: Event) {
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
}

// #[cfg(test)]
// mod tests {
// use color_eyre::{eyre::WrapErr, Report};
// use crossterm::event::{
//     DisableMouseCapture, EnableMouseCapture, Event as cEvent, KeyCode, KeyEvent,
// };
// use crossterm::terminal::{
//     disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
// };
// use std::io;
// use tui::backend::CrosstermBackend;
// use tui::Terminal;

// fn reset_terminal() -> std::result::Result<(), Box<dyn std::error::Error>> {
//     disable_raw_mode()?;
//     crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

//     Ok(())
// }

// fn main() -> Result<(), Report> {
//     let runtime = tokio::runtime::Runtime::new().wrap_err("failed to initialize tokio")?;
//     runtime
//         .block_on(runtime.spawn(async move { async_main().await.unwrap() }))
//         .wrap_err("Err joining")
// }

// // #[test]
// async fn async_main() -> Result<(), Report> {
//     let stdout = io::stdout();
//     let mut stdout = stdout.lock();

//     {
//         let original_hook = std::panic::take_hook();

//         std::panic::set_hook(Box::new(move |panic| {
//             reset_terminal().unwrap();
//             original_hook(panic);
//         }));
//     }

//     let mut tree_edit = TreeEdit::new("TestTreeEdit")
//         .tab(
//             Tab::new("Tab 1").branch(Tree::new("Tree 1")).branch(
//                 Args::new("Args 1")
//                     .names(["Name 1", "Name 2"])
//                     .columns(["Column 1", "Column 2"])
//                     .value("Name 2", "Column 1", true)
//                     .value("Name 1", "Column 2", 35.0 as f32)
//                     .value(
//                         "Name 2",
//                         "Column 2",
//                         Args::new("Tree arg 2")
//                             .names(["Name 11", "Name 12"])
//                             .columns(["Column 11", "Column 12"])
//                             .value("Name 12", "Column 12", true),
//                     )
//                     .value(
//                         "Name 1",
//                         "Column 1",
//                         Tree::new("Tree arg 1").branch(Tree::new("Test")),
//                     ),
//             ),
//         )
//         .tab(Tab::new("Tab 2"))
//         .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
//     tree_edit.state.position = vec![
//         Node::Tree("Tab 1".to_string()),
//         Node::Tree("Args 1".to_string()),
//         Node::Args("Name 1".to_string(), 0),
//         // Node::Tree("Tree arg 2".to_string()),
//     ];
//     // tree_edit.state.column = 1;

//     enable_raw_mode()?;
//     crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
//     let backend = CrosstermBackend::new(stdout);
//     let mut term = Terminal::new(backend)?;

//     loop {
//         term.draw(|f| {
//             f.render_widget(tree_edit.widget(), f.size());
//         })
//         .unwrap();
//         match crossterm::event::read().unwrap() {
//             cEvent::Key(KeyEvent {
//                 code: KeyCode::Esc, ..
//             }) => break,
//             _ => (),
//         }
//     }

//     reset_terminal().unwrap();

//     Ok(())
// }
// }
