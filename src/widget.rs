use tui::layout::{Constraint, Direction, Layout};
use tui::text::Spans;
use tui::widgets::{Block, Borders, Tabs, Widget};

use crate::state::State;

use super::TreeEdit;

pub struct Drawer<'a>(&'a TreeEdit<'a>);

impl<'a> Drawer<'a> {
    pub fn new(tree_edit: &'a TreeEdit<'a>) -> Self {
        Self(tree_edit)
    }
}

pub trait DrawerRef<'a> {
    fn render(
        &'a self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: (&State, &mut usize),
    );
}

impl<'a> Widget for Drawer<'a> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        if area.area() == 0 {
            return;
        }

        let tree_edit = self.0;
        let tab_titles = {
            tree_edit
                .tabs
                .iter()
                .map(|tab| tab.get_name().into())
                .collect::<Vec<Spans<'a>>>()
        };

        // buf.set_style(area, tree_edit.state.style);

        let block = Block::default()
            .title(tree_edit.title.clone())
            // .style(tree_edit.state.style)
            // .border_type(tui::widgets::BorderType::Thick)
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        tui::widgets::Widget::render(block, area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(3)])
            .split(inner_area);

        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::BOTTOM))
            .style(tree_edit.state.style)
            .highlight_style(tree_edit.state.highlight_style);
        let tabs = if let Some(index) = tree_edit.get_index_tab() {
            tabs.select(index)
        } else {
            tabs
        };
        tui::widgets::Widget::render(tabs, chunks[0], buf);

        tree_edit
            .get_tab()
            .map(|tab| tab.render(chunks[1], buf, (&tree_edit.state, &mut 1)));
    }
}
