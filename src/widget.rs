use tui::buffer::Buffer;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::text::Spans;
use tui::widgets::{Block, Borders, Tabs, Widget};

use crate::state::State;

use super::TreeEdit;

pub struct Drawer<'a, 'b>(&'b TreeEdit<'a>);

impl Drawer<'_, '_> {
    pub fn new<'a, 'b>(tree_edit: &'b TreeEdit<'a>) -> Drawer<'a, 'b> {
        Drawer(tree_edit)
    }
}

pub trait DrawerRef {
    fn render(
        &self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: (&State, &mut usize),
    ) -> u16;
}

impl Widget for Drawer<'_, '_> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        if area.area() == 0 {
            return;
        }

        let tab_titles = {
            self.0
                .tabs
                .iter()
                .map(|(tab_name, _)| tab_name.clone().into())
                .collect::<Vec<Spans>>()
        };
        let path_segments = {
            self.0
                .position()
                .iter()
                .filter_map(|node| match node {
                    crate::Node::Tree(name) => Some(name.clone()),
                    crate::Node::Args {
                        name: row, column, ..
                    } => {
                        let (_, args) = self.0.get_current_tab()?;
                        let args = args.as_args()?;
                        let column = args.get_columns_raw().get(*column)?;
                        Some(format!("({row}, {column})"))
                    }
                })
                .map(Into::into)
                .collect::<Vec<Spans>>()
        };

        let block = Block::default()
            .title(self.0.title.clone())
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(3),
                Constraint::Length(2),
            ])
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

        let path = Tabs::new(path_segments)
            .block(Block::default().borders(Borders::TOP))
            .style(self.0.state.style)
            .divider(">")
            .highlight_style(if self.0.state.input.is_some() {
                self.0.state.style
            } else {
                self.0.state.highlight_style
            });
        let path = {
            let len = self.0.position().len();
            if len > 0 {
                path.select(len - 1)
            } else {
                path
            }
        };

        tabs.render(chunks[0], buf);
        path.render(chunks[2], buf);

        let Some((_, tab)) = self.0.get_current_tab() else {
            return;
        };
        let rect = Rect {
            x: 0,
            y: 0,
            width: 1 << 9,
            height: chunks[1].height,
        };
        let mut temp_buf = Buffer::empty(rect);
        let width = tab.render(rect, &mut temp_buf, (&self.0.state, &mut 1));

        let rect = chunks[1];
        let offset = width.saturating_sub(rect.width + 1);
        for y in 0..rect.height {
            let y1 = rect.y + y;
            for x in 1..width {
                let x1 = rect.x + x;
                *buf.get_mut(x1, y1) = temp_buf.get(offset + x, y).clone()
            }
        }
    }
}
