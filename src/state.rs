use std::fmt::Debug;

use crate::{array::Array, Branch, Value};
use tui::style::Style;

type BranchItem<'a, 'b> = (&'b String, &'b Branch<'a>);
type BranchItemMut<'a, 'b> = (&'b String, &'b mut Branch<'a>);

#[derive(Clone, Debug)]
pub enum Node {
    Tree(String),
    Args(String, usize),
}
impl Node {
    pub fn is_tree(&self) -> bool {
        matches!(self, Self::Tree(_))
    }
    pub fn is_args(&self) -> bool {
        matches!(self, Self::Args(_, _))
    }

    pub fn as_tree(&self) -> Option<&String> {
        if let Self::Tree(name) = self {
            Some(name)
        } else {
            None
        }
    }
    pub fn as_args(&self) -> Option<(&String, &usize)> {
        if let Self::Args(name, col) = self {
            Some((name, col))
        } else {
            None
        }
    }

    pub fn text(&self) -> &String {
        self.as_tree()
            .or_else(|| self.as_args().map(|(text, _)| text))
            .unwrap()
    }
    pub fn text_mut(&mut self) -> &mut String {
        match self {
            Node::Tree(t) | Node::Args(t, _) => t,
        }
    }
    fn change_text(&mut self, text: String) {
        *match self {
            Node::Tree(tree) => tree,
            Node::Args(args, _) => args,
        } = text
    }
    fn inc_index(&mut self, max: usize) {
        if let Self::Args(_, col) = self {
            *col = max.min(*col + 1);
        }
    }
    fn dec_index(&mut self) -> bool {
        if let Self::Args(_, col) = self {
            if *col == 0 {
                return true;
            } else {
                *col -= 1;
            }
        }

        false
    }
}
impl From<&Branch<'_>> for Node {
    fn from(branch: &Branch) -> Self {
        match branch {
            Branch::Args(_) => Self::Args(
                branch
                    .get_list()
                    .first()
                    .unwrap_or(&String::default())
                    .clone(),
                0,
            ),
            Branch::Tree(_) | Branch::Array(_) => Self::Tree(
                branch
                    .get_list()
                    .first()
                    .unwrap_or(&String::default())
                    .clone(),
            ),
        }
    }
}
#[derive(Default, Clone)]
pub struct State {
    pub position: Vec<Node>,
    pub input: Option<String>,
    pub style: Style,
    pub highlight_style: Style,
}
impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("position", &self.position)
            .field("input", &self.input)
            .finish()
    }
}
impl State {
    pub fn index_tab(&self, tabs: &crate::Branches) -> Option<usize> {
        tabs.iter().position(|(tab_name, _)| {
            self.position
                .get(0)
                .and_then(|node| node.as_tree().map(|name| name == tab_name))
                .unwrap_or(false)
        })
    }

    pub fn node(&self, level: usize) -> Option<&Node> {
        self.position.get(level)
    }

    pub fn current_tab<'a, 'b>(
        &self,
        tabs: &'b mut crate::Branches<'a>,
    ) -> Option<BranchItemMut<'a, 'b>> {
        self.index_tab(tabs)
            .and_then(|index| tabs.iter_mut().nth(index))
    }
    fn next_tab<'a, 'b>(&self, tabs: &'b crate::Branches<'a>) -> Option<BranchItem<'a, 'b>> {
        tabs.iter().nth(self.index_tab(tabs).unwrap_or(0) + 1)
    }
    fn previous_tab<'a, 'b>(&self, tabs: &'b crate::Branches<'a>) -> Option<BranchItem<'a, 'b>> {
        self.index_tab(tabs).and_then(|index| {
            tabs.iter()
                .nth(if index == 0 { usize::MAX } else { index - 1 })
        })
    }

    fn branch<'a, 'b>(
        &self,
        tabs: &'b mut crate::Branches<'a>,
        offset: usize,
    ) -> Option<&'b mut Branch<'a>> {
        self.current_tab(tabs).and_then(|(_, branch)| {
            Self::incise_position(
                branch,
                self.position.iter().skip(1).rev().skip(offset + 1).rev(),
            )
        })
    }
    #[inline]
    fn current_branch<'a, 'b>(
        &self,
        tabs: &'b mut crate::Branches<'a>,
    ) -> Option<&'b mut Branch<'a>> {
        self.branch(tabs, 0)
    }

    fn next_item_impl<'a>(&self, iter: impl Iterator<Item = &'a String>) -> Option<String> {
        iter.skip_while(|&name| {
            self.position
                .last()
                .map_or(true, |node| *name != *node.text())
        })
        .nth(1)
        .cloned()
    }
    fn next_item(&self, tabs: &mut crate::Branches) -> Option<String> {
        self.current_branch(tabs)
            .and_then(|branch| self.next_item_impl(branch.get_list().iter()))
    }
    fn previous_item(&self, tabs: &mut crate::Branches) -> Option<String> {
        self.current_branch(tabs)
            .and_then(|branch| self.next_item_impl(branch.get_list().iter().rev()))
    }

    fn value<'a, 'b>(
        &self,
        tabs: &'b mut crate::Branches<'a>,
        offset: usize,
    ) -> Option<&'b mut Value<'a>> {
        self.branch(tabs, offset)
            .and_then(|branch| branch.as_args_mut())
            .zip(
                self.position
                    .iter()
                    .nth_back(offset)
                    .and_then(|node| node.as_args()),
            )
            .and_then(|(args, (name, column))| args.get_value_by_cindex_mut(name, *column))
    }
    #[inline]
    fn current_value<'a, 'b>(
        &self,
        tabs: &'b mut crate::Branches<'a>,
    ) -> Option<&'b mut Value<'a>> {
        self.value(tabs, 0)
    }

    fn incise_position<'a, 'b, 'n>(
        branch: &'b mut Branch<'a>,
        mut nodes: impl Iterator<Item = &'n Node>,
    ) -> Option<&'b mut Branch<'a>> {
        if let Some(node) = nodes.next() {
            if let Some(branch) = match branch {
                Branch::Args(args) => args
                    .get_value_by_cindex_mut(
                        node.text(),
                        node.as_args().map_or(usize::MAX, |(_, index)| *index),
                    )
                    .and_then(|value| value.as_struct_mut()),
                Branch::Tree(tree) | Branch::Array(Array { tree, .. }) => {
                    tree.get_branches_mut().get_mut(node.text())
                }
            } {
                Self::incise_position(branch, nodes)
            } else {
                None
            }
        } else {
            Some(branch)
        }
    }

    pub fn transition(&mut self, event: crate::Event, tabs: &mut crate::Branches) {
        if self.input.is_some() {
            self.enter_handler(tabs, event);
            return;
        }

        use crate::Event::*;
        match event {
            NextTab | PreviousTab => self.tab_handler(tabs, event == NextTab),
            NextItem | PreviousItem => self.item_handler(tabs, event == NextItem),
            NextLevel | Enter => {
                if event == Enter {
                    if self.enter_handler(tabs, Enter) {
                        self.next_level_handler(tabs, event == NextLevel)
                    }
                } else {
                    self.next_level_handler(tabs, event == NextLevel)
                }
            }
            PreviousLevel => {
                if (self.position.len() > 2
                    || self
                        .position
                        .last()
                        .and_then(|node| node.as_args())
                        .and_then(|(_, index)| (*index > 0).then_some(()))
                        .is_some())
                    && self
                        .current_branch(tabs)
                        .and_then(|branch| match branch {
                            Branch::Args(_) => Some(()),
                            Branch::Tree(_) | Branch::Array(_) => None,
                        })
                        .map_or(Some(()), |_| {
                            self.position
                                .last_mut()
                                .and_then(|node| node.dec_index().then_some(()))
                        })
                        .is_some()
                {
                    self.position.pop();
                }
            }
            Delete => {
                let Some(node) = self.position.last().map(|n| n.text()) else {
                    return;
                };

                if let Some(array) = self
                    .value(tabs, 1)
                    .and_then(|v| v.as_array_mut())
                    .filter(|a| !a.get_branches().is_empty())
                {
                    if let Some(current) = array.remove(node) {
                        *self.position.last_mut().map(|n| n.text_mut()).unwrap() = current
                    } else {
                        self.position.pop();
                    }
                }
            }
            _ => (),
        }
    }

    fn tab_handler(&mut self, tabs: &mut crate::Branches, is_next: bool) {
        if let Some((name, branch)) = if is_next {
            self.next_tab(tabs).or_else(|| tabs.front())
        } else {
            self.previous_tab(tabs).or_else(|| tabs.back())
        } {
            self.position.clear();
            self.position.push(Node::Tree(name.clone()));
            self.position.push(branch.into())
        };
        self.transition(crate::Event::PreviousItem, tabs)
    }

    fn item_handler(&mut self, tabs: &mut crate::Branches, is_next: bool) {
        if is_next {
            self.next_item(tabs)
        } else {
            self.previous_item(tabs)
        }
        .map(|text| self.position.last_mut().map(|node| node.change_text(text)));
    }

    fn next_level_handler(&mut self, tabs: &mut crate::Branches, is_next: bool) {
        let Some(branch) = self.current_branch(tabs) else {
            return;
        };

        let list = branch.get_list();
        let res = match branch {
            Branch::Args(args) => 'args: {
                let columns = args.get_columns();
                if columns.len() > 1 && is_next {
                    break 'args self.position.last().map(|node| {
                        let mut node = node.clone();
                        node.inc_index(columns.len() - 1);
                        (node, true)
                    });
                }
                list.into_iter()
                    .position(|name| {
                        self.position
                            .last()
                            .map_or(true, |node| name == *node.text())
                    })
                    .and_then(|name| {
                        let index = (!is_next)
                            .then(|| {
                                self.position
                                    .last()
                                    .and_then(|node| node.as_args().map(|(_, index)| *index))
                            })
                            .flatten();
                        args.get_value_by_indexes(name, index.unwrap_or(0))
                    })
                    .and_then(|v| {
                        v.as_struct()
                            .filter(|_| v.as_array().map_or(true, |a| !a.branches.is_empty()))
                            .map(Into::into)
                    })
                    .map(|node| (node, false))
            }
            Branch::Tree(tree) | Branch::Array(Array { tree, .. }) => tree
                .get_branches()
                .into_iter()
                .find_map(|(name, branch)| {
                    self.position.last().and_then(|node| {
                        (name == node.text() && !branch.is_empty()).then_some(branch)
                    })
                })
                .map(|current| (Node::from(current), false)),
        };
        let Some((node, replace)) = res else {
            return;
        };

        if replace {
            self.position.pop();
        }
        self.position.push(node);
    }

    fn enter_handler(&mut self, tabs: &mut crate::Branches, event: crate::Event) -> bool {
        if let Some(value) = self.current_value(tabs) {
            let mut to_check = false;
            let check = value.check();

            if value.is_bool() {
                if let Some(value) = value.as_bool_mut() {
                    *value = !*value
                };
            } else if let Some(arr) = value.as_array_mut() {
                arr.insert_default(None);
                return false;
            } else if let Some(text) = value.as_text() {
                use crate::Event::*;
                let mut text = text.lock().unwrap();
                if self.input.is_none() && event == Enter {
                    self.input = text.lines().get(0).cloned();
                    to_check = true;
                } else if let Some(saved) = &self.input {
                    match event {
                        NextLevel => text.move_cursor(tui_textarea::CursorMove::Forward),
                        PreviousLevel => text.move_cursor(tui_textarea::CursorMove::Back),
                        Enter => {
                            if check {
                                self.input = None;
                                text.move_cursor(tui_textarea::CursorMove::End);
                            }
                        }
                        Cancel => {
                            text.move_cursor(tui_textarea::CursorMove::End);
                            text.delete_line_by_head();
                            text.insert_str(saved);
                            text.move_cursor(tui_textarea::CursorMove::End);
                            self.input = None;
                        }
                        Char(sym) => {
                            text.insert_char(sym);
                            to_check = true;
                        }
                        Backspace => {
                            text.delete_char();
                            to_check = true;
                        }
                        _ => (),
                    }
                }
            }

            if to_check {
                let res = value.check();
                if let Some(text) = value.as_text() {
                    let mut text = text.lock().unwrap();
                    if !res {
                        text.set_style(Style::default().fg(tui::style::Color::Red));
                    } else {
                        text.set_style(Style::default().fg(tui::style::Color::Green));
                    }
                };
            }
        }
        true
    }
}
