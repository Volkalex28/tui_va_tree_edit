use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget};

use crate::state::State;
use crate::widget::DrawerRef;
use crate::Branch;

#[derive(Debug, Default, Clone)]
pub struct Tree<'a> {
    pub(crate) branches: crate::Branches<'a>,
}
impl<'a> Tree<'a> {
    pub fn branch(mut self, branch_name: impl ToString, branch: impl Into<Branch<'a>>) -> Self {
        self.branches.insert(branch_name.to_string(), branch.into());
        self
    }

    pub fn get_branches(&self) -> &crate::Branches<'a> {
        &self.branches
    }
    pub fn get_branches_mut(&mut self) -> &mut crate::Branches<'a> {
        &mut self.branches
    }
}
impl DrawerRef for Tree<'_> {
    fn render(
        &self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: (&State, &mut usize),
    ) -> u16 {
        if self.branches.is_empty() || *state.1 > state.0.position.len() {
            return 0;
        }
        let items = {
            self.branches
                .iter()
                .map(|(name, _)| name)
                .collect::<Vec<&String>>()
        };
        let width = items
            .iter()
            .map(|span| span.len() as u16)
            .fold(0, |max, width| max.max(width + 5));
        let constrains = vec![Constraint::Length(width), Constraint::Min(3)];
        let mut list_state = ListState::default();

        let current =
            state.0.node(*state.1).and_then(|current| {
                current.as_tree().and_then(|current| {
                    items.iter().enumerate().find_map(|(number, &text)| {
                        if text == current {
                            Some(number)
                        } else {
                            None
                        }
                    })
                })
            });

        let list = List::new(
            items
                .into_iter()
                .map(|text| ListItem::new(text.clone()))
                .collect::<Vec<ListItem>>(),
        )
        .block(Block::default().borders(Borders::RIGHT))
        .highlight_symbol(if *state.1 + 1 == state.0.position.len() {
            ">>"
        } else {
            "  "
        })
        .repeat_highlight_symbol(true)
        .style(state.0.style)
        .highlight_style(
            if *state.1 < state.0.position.len() && state.0.input.is_none() {
                state.0.highlight_style
            } else {
                state.0.style
            },
        );
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constrains)
            .split(area);

        list_state.select(Some(current.unwrap_or(usize::MAX)));
        list.render(chunks[0], buf, &mut list_state);
        if let Some(index) = current {
            *state.1 += 1;
            if let Some((_, branch)) = self.branches.iter().nth(index) {
                return chunks[0].width + branch.render(chunks[1], buf, state);
            }
        };
        chunks[0].width
    }
}
