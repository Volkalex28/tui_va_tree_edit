use tui::style::Style;

use crate::{Branch, Value};

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

    pub fn tree(&self) -> Option<&String> {
        if let Self::Tree(name) = self {
            Some(name)
        } else {
            None
        }
    }
    pub fn args(&self) -> Option<(&String, &usize)> {
        if let Self::Args(name, col) = self {
            Some((name, col))
        } else {
            None
        }
    }

    pub fn text(&self) -> &String {
        self.tree()
            .or_else(|| self.args().map(|(text, _)| text))
            .unwrap()
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
impl<'a> From<&Branch<'a>> for Node {
    fn from(branch: &Branch<'a>) -> Self {
        match branch {
            Branch::Args(_) => Self::Args(
                branch
                    .get_list()
                    .first()
                    .unwrap_or(&String::default())
                    .clone(),
                0,
            ),
            Branch::Tree(_) => Self::Tree(
                branch
                    .get_list()
                    .first()
                    .unwrap_or(&String::default())
                    .clone(),
            ),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct State {
    pub position: Vec<Node>,
    pub input: Option<String>,
    pub style: Style,
    pub highlight_style: Style,
}
impl State {
    pub fn index_tab<'a>(&self, tabs: &crate::Branches<'a>) -> Option<usize> {
        tabs.iter().position(|(tab_name, _)| {
            self.position
                .get(0)
                .and_then(|node| node.tree().map(|name| name == tab_name))
                .unwrap_or(false)
        })
    }

    pub fn node(&self, level: usize) -> Option<&Node> {
        self.position.get(level)
    }

    pub fn current_tab<'a, 'b>(
        &self,
        tabs: &'b mut crate::Branches<'a>,
    ) -> Option<crate::BranchItemMut<'a, 'b>> {
        self.index_tab(tabs)
            .and_then(|index| tabs.iter_mut().nth(index))
    }
    fn next_tab<'a, 'b>(&self, tabs: &'b crate::Branches<'a>) -> Option<crate::BranchItem<'a, 'b>> {
        tabs.iter().nth(self.index_tab(tabs).unwrap_or(0) + 1)
    }
    fn previous_tab<'a, 'b>(
        &self,
        tabs: &'b crate::Branches<'a>,
    ) -> Option<crate::BranchItem<'a, 'b>> {
        self.index_tab(tabs).and_then(|index| {
            tabs.iter()
                .nth(if index == 0 { usize::MAX } else { index - 1 })
        })
    }

    fn current_branch<'a, 'b>(
        &self,
        tabs: &'b mut crate::Branches<'a>,
    ) -> Option<&'b mut Branch<'a>> {
        self.current_tab(tabs).and_then(|(_, branch)| {
            Self::incise_position(branch, self.position.iter().skip(1).rev().skip(1).rev())
        })
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
    fn next_item<'a>(&self, tabs: &mut crate::Branches<'a>) -> Option<String> {
        self.current_branch(tabs)
            .and_then(|branch| self.next_item_impl(branch.get_list().iter()))
    }
    fn previous_item<'a>(&self, tabs: &mut crate::Branches<'a>) -> Option<String> {
        self.current_branch(tabs)
            .and_then(|branch| self.next_item_impl(branch.get_list().iter().rev()))
    }

    fn current_value<'a, 'b>(
        &self,
        tabs: &'b mut crate::Branches<'a>,
    ) -> Option<&'b mut Value<'a>> {
        self.current_branch(tabs)
            .and_then(|branch| branch.args_mut())
            .zip(self.position.last().and_then(|node| node.args()))
            .and_then(|(args, (name, column))| args.get_value_by_cindex_mut(name, *column))
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
                        node.args().map_or(usize::MAX, |(_, index)| *index),
                    )
                    .and_then(|value| value.as_struct_mut()),
                Branch::Tree(tree) => tree.get_branches_mut().get_mut(node.text()),
            } {
                Self::incise_position(branch, nodes)
            } else {
                None
            }
        } else {
            Some(branch)
        }
    }

    pub fn transition<'a>(&mut self, event: crate::Event, tabs: &mut crate::Branches<'a>) {
        if self.input.is_some() {
            self.enter_handler(tabs, event)
        } else {
            use crate::Event::*;
            match event {
                NextTab | PreviousTab => {
                    if let Some((name, branch)) = if event == NextTab {
                        self.next_tab(tabs).or_else(|| tabs.front())
                    } else {
                        self.previous_tab(tabs).or_else(|| tabs.back())
                    } {
                        self.position.clear();
                        self.position.push(Node::Tree(name.clone()));
                        self.position.push(branch.into())
                    };
                    self.transition(PreviousItem, tabs);
                }
                NextItem | PreviousItem => {
                    if event == NextItem {
                        self.next_item(tabs)
                    } else {
                        self.previous_item(tabs)
                    }
                    .map(|text| self.position.last_mut().map(|node| node.change_text(text)));
                }
                NextLevel | Enter => {
                    if event == Enter {
                        self.enter_handler(tabs, Enter);
                    }
                    if let Some((node, replace)) = self.current_branch(tabs).and_then(|branch| {
                        let list = branch.get_list();
                        match branch {
                            Branch::Args(args) => {
                                let columns = args.get_columns();
                                if columns.len() > 1 && event != Enter {
                                    self.position
                                        .last()
                                        .map(|node| {
                                            let mut node = node.clone();
                                            node.inc_index(columns.len() - 1);
                                            node
                                        })
                                        .map(|node| (node, true))
                                } else {
                                    list.into_iter()
                                        .position(|name| {
                                            self.position
                                                .last()
                                                .map_or(true, |node| name == *node.text())
                                        })
                                        .and_then(|name| {
                                            args.get_value_by_indexes(
                                                name,
                                                if event == Enter {
                                                    self.position
                                                        .last()
                                                        .and_then(|node| {
                                                            node.args().map(|(_, index)| *index)
                                                        })
                                                        .unwrap_or(0)
                                                } else {
                                                    0
                                                },
                                            )
                                        })
                                        .and_then(|value| value.as_struct())
                                        .map(|branch| (Node::from(branch), false))
                                }
                            }
                            Branch::Tree(tree) => tree
                                .get_branches()
                                .into_iter()
                                .find_map(|(name, branch)| {
                                    self.position.last().and_then(|node| {
                                        if name == node.text() && !branch.is_empty() {
                                            Some(branch)
                                        } else {
                                            None
                                        }
                                    })
                                })
                                .map(|current| (Node::from(current), false)),
                        }
                    }) {
                        if replace {
                            self.position.pop();
                        }
                        self.position.push(node)
                    }
                }
                PreviousLevel => {
                    if (self.position.len() > 2
                        || self
                            .position
                            .last()
                            .and_then(|node| node.args())
                            .and_then(|(_, index)| (*index > 0).then_some(()))
                            .is_some())
                        && self
                            .current_branch(tabs)
                            .and_then(|branch| match branch {
                                Branch::Args(_) => Some(()),
                                Branch::Tree(_) => None,
                            })
                            .map_or(Some(()), |_| {
                                self.position
                                    .last_mut()
                                    .and_then(|node| node.dec_index().then_some(()))
                            })
                            .is_some()
                    {
                        self.position.pop();
                    };
                }
                _ => (),
            }
        }
    }

    fn enter_handler<'a>(&mut self, tabs: &mut crate::Branches<'a>, event: crate::Event) {
        if let Some(value) = self.current_value(tabs) {
            let mut to_check = false;
            let check = value.check();

            if value.is_bool() {
                if let Some(value) = value.as_bool_mut() {
                    *value = !*value
                };
            } else if let Some(text) = value.as_text_mut() {
                use crate::Event::*;
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
                if let Some(text) = value.as_text_mut() {
                    if !res {
                        text.set_style(Style::default().fg(tui::style::Color::Red));
                    } else {
                        text.set_style(Style::default().fg(tui::style::Color::Green));
                    }
                };
            }
        }
    }
}
