use tui::layout::{Constraint, Direction, Layout};
use tui::text::Spans;
use tui::widgets::{Block, Borders, Tabs, Widget};

use crate::state::State;

use super::TreeEdit;

pub struct Drawer<'a, 'b>(&'b TreeEdit<'a>);

impl<'a, 'b> Drawer<'a, 'b> {
    pub fn new(tree_edit: &'b TreeEdit<'a>) -> Self {
        Self(tree_edit)
    }
}

pub trait DrawerRef<'a> {
    fn render(
        &self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: (&State, &mut usize),
    );
}

impl<'a, 'b> Widget for Drawer<'a, 'b> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        if area.area() == 0 {
            return;
        }

        let tab_titles = {
            self.0
                .tabs
                .iter()
                .map(|(tab_name, _)| tab_name.clone().into())
                .collect::<Vec<Spans<'a>>>()
        };

        let block = Block::default()
            .title(self.0.title.clone())
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(3)])
            .split(inner_area);

        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::BOTTOM))
            .style(self.0.state.style)
            .highlight_style(if self.0.state.input.is_some() {
                self.0.state.style
            } else {
                self.0.state.highlight_style
            });
        let tabs = if let Some(index) = self.0.get_index_tab() {
            tabs.select(index)
        } else {
            tabs
        };
        tabs.render(chunks[0], buf);

        self.0
            .get_current_tab()
            .map(|(_, tab)| tab.render(chunks[1], buf, (&self.0.state, &mut 1)));
    }
}
