use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget};

use crate::argument::Args;
use crate::state::State;
use crate::widget::DrawerRef;

#[derive(Clone)]
pub enum Branch<'a> {
    Args(Args<'a>),
    Tree(Tree<'a>),
}

impl<'a> Branch<'a> {
    pub fn is_args(&self) -> bool {
        matches!(self, Self::Args(_))
    }
    pub fn is_tree(&self) -> bool {
        matches!(self, Self::Tree(_))
    }
    pub fn args(&self) -> Option<&Args<'a>> {
        if let Self::Args(args) = &self {
            Some(args)
        } else {
            None
        }
    }
    pub fn tree(&self) -> Option<&Tree<'a>> {
        if let Self::Tree(tree) = &self {
            Some(tree)
        } else {
            None
        }
    }
    pub fn args_mut(&mut self) -> Option<&mut Args<'a>> {
        if let Self::Args(args) = self {
            Some(args)
        } else {
            None
        }
    }
    pub fn tree_mut(&mut self) -> Option<&mut Tree<'a>> {
        if let Self::Tree(tree) = self {
            Some(tree)
        } else {
            None
        }
    }

    // pub fn get_name(&self) -> String {
    //     match self {
    //         Branch::Args(args) => args.get_name(),
    //         Branch::Tree(tree) => tree.get_name(),
    //     }
    // }
    pub fn get_list(&self) -> Vec<String> {
        match self {
            Branch::Args(args) => args
                .get_names()
                .into_iter()
                .map(|span| span.content.to_string())
                .collect(),
            Branch::Tree(tree) => tree
                .branches
                .iter()
                .map(|(name, _)| name.clone())
                .collect(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.get_list().len() == 0
    }
}
impl<'a> DrawerRef<'a> for Branch<'a> {
    fn render(
        &self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: (&State, &mut usize),
    ) {
        match self {
            Branch::Args(args) => args.render(area, buf, state),
            Branch::Tree(tree) => tree.render(area, buf, state),
        }
    }
}
impl<'a> From<Tree<'a>> for Branch<'a> {
    fn from(tree: Tree<'a>) -> Self {
        Self::Tree(tree)
    }
}
impl<'a> From<Args<'a>> for Branch<'a> {
    fn from(args: Args<'a>) -> Self {
        Self::Args(args)
    }
}

#[derive(Default, Clone)]
pub struct Tree<'a> {
    branches: crate::Branches<'a>,
}
impl<'a> Tree<'a> {

    pub fn branch(mut self, branch_name: impl Into<String>, branch: impl Into<Branch<'a>>) -> Self {
        self.branches.insert(branch_name.into(), branch.into());
        self
    }

    pub fn get_branches(&self) -> &crate::Branches<'a> {
        &self.branches
    }
    pub fn get_branches_mut(&mut self) -> &mut crate::Branches<'a> {
        &mut self.branches
    }
}
impl<'a> DrawerRef<'a> for Tree<'a> {
    fn render(
        &self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: (&State, &mut usize),
    ) {
        if self.branches.len() == 0 || *state.1 > state.0.position.len() {
            return;
        }
        let items = {
            self.branches
                .iter()
                .map(|(name, _)| name)
                .collect::<Vec<&String>>()
        };
        let constrains = vec![
            Constraint::Length(
                items
                    .iter()
                    .map(|span| span.len() as u16)
                    .fold(0, |max, width| max.max(width + 5)),
            ),
            Constraint::Min(3),
        ];
        let mut list_state = ListState::default();

        let current =
            state.0.node(*state.1).and_then(|current| {
                current.tree().and_then(|current| {
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
                .collect::<Vec<ListItem<'a>>>(),
        )
        .block(Block::default().borders(Borders::RIGHT))
        .highlight_symbol(if *state.1 + 1 == state.0.position.len() {
            ">>"
        } else {
            "  "
        })
        .repeat_highlight_symbol(true)
        .style(state.0.style)
        .highlight_style(if *state.1 + 1 <= state.0.position.len() && state.0.input.is_none() {
            state.0.highlight_style
        } else {
            state.0.style
        });
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constrains)
            .split(area);

        list_state.select(Some(current.unwrap_or(usize::MAX)));
        list.render(chunks[0], buf, &mut list_state);
        current.map(|index| {
            *state.1 += 1;
            self.branches
                .iter()
                .skip(index)
                .next()
                .map(|(_, branch)| branch.render(chunks[1], buf, state));
        });
    }
}
