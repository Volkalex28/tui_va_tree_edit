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

    fn text(&self) -> &String {
        self.tree()
            .or(self.args().and_then(|(text, _)| Some(text)))
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
    fn dec_index(&mut self) -> Option<()> {
        if let Self::Args(_, col) = self {
            if *col == 0 {
                Some(())
            } else {
                *col -= 1;
                None
            }
        } else {
            None
        }
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
    pub input: bool,
    pub style: Style,
    pub highlight_style: Style,
}
impl State {
    pub fn index_tab<'a>(&self, tabs: &Vec<Branch<'a>>) -> Option<usize> {
        tabs.iter().position(|tab| {
            self.position
                .get(0)
                .and_then(|node| node.tree().and_then(|name| Some(*name == tab.get_name())))
                .unwrap_or(false)
        })
    }

    pub fn node(&self, level: usize) -> Option<&Node> {
        self.position.get(level)
    }

    fn current_tab<'a, 'b>(&mut self, tabs: &'b mut Vec<Branch<'a>>) -> Option<&'b mut Branch<'a>> {
        self.index_tab(tabs).and_then(|index| tabs.get_mut(index))
    }
    fn next_tab<'a, 'b>(&self, tabs: &'b Vec<Branch<'a>>) -> Option<&'b Branch<'a>> {
        self.index_tab(tabs).and_then(|index| tabs.get(index + 1))
    }
    fn previous_tab<'a, 'b>(&self, tabs: &'b Vec<Branch<'a>>) -> Option<&'b Branch<'a>> {
        self.index_tab(tabs)
            .and_then(|index| tabs.get(if index == 0 { usize::MAX } else { index - 1 }))
    }

    fn current_branch<'a, 'b>(
        &mut self,
        tabs: &'b mut Vec<Branch<'a>>,
    ) -> Option<&'b mut Branch<'a>> {
        self.current_tab(tabs).and_then(|branch| {
            Self::incise_position(branch, self.position.iter().skip(1).rev().skip(1).rev())
        })
    }
    fn next_item_impl<'a>(&self, iter: impl Iterator<Item = &'a String> + 'a) -> Option<String> {
        iter.skip_while(|&name| {
            self.position
                .last()
                .map_or(true, |node| *name != *node.text())
        })
        .skip(1)
        .next()
        .map(|text| text.clone())
    }
    fn next_item<'a, 'b>(&mut self, tabs: &'b mut Vec<Branch<'a>>) -> Option<String> {
        self.current_branch(tabs)
            .and_then(|branch| self.next_item_impl(branch.get_list().iter()))
    }
    fn previous_item<'a, 'b>(&mut self, tabs: &'b mut Vec<Branch<'a>>) -> Option<String> {
        self.current_branch(tabs)
            .and_then(|branch| self.next_item_impl(branch.get_list().iter().rev()))
    }

    fn current_value<'a, 'b>(
        &mut self,
        tabs: &'b mut Vec<Branch<'a>>,
    ) -> Option<&'b mut Value<'a>> {
        self.current_branch(tabs)
            .and_then(|branch| branch.args_mut())
            .zip(self.position.last().and_then(|node| node.args()))
            .and_then(|(args, (name, index))| {
                args.get_columns()
                    .get(*index)
                    .map(|column| (args, name, column.content.to_string()))
            })
            .and_then(|(args, name, column)| args.get_value_mut(name, &column))
    }

    fn incise_position<'a, 'b, 'n>(
        branch: &'b mut Branch<'a>,
        mut nodes: impl Iterator<Item = &'n Node>,
    ) -> Option<&'b mut Branch<'a>> {
        if let Some(node) = nodes.next() {
            let list = branch.get_list();
            if let Some(branch) = match branch {
                Branch::Args(args) => args
                    .get_value_mut(
                        node.text(),
                        &args
                            .get_columns()
                            .get(node.args().map_or(usize::MAX, |(_, index)| *index))
                            .map_or("".to_string(), |span| span.content.to_string()),
                    )
                    .and_then(|value| value.as_struct_mut()),
                Branch::Tree(tree) => tree
                    .get_branches_mut()
                    .get_mut(
                        list.into_iter()
                            .position(|name| name == *node.text())
                            .unwrap_or(0),
                    )
                    .map(|branch| branch),
            } {
                Self::incise_position(branch, nodes)
            } else {
                None
            }
        } else {
            Some(branch)
        }
    }

    pub fn transition<'a, 'b>(&mut self, event: crate::Event, tabs: &'b mut Vec<Branch<'a>>) {
        if self.input {
            self.enter_handler(tabs, event)
        } else {
            use crate::Event::*;
            match event {
                NextTab | PreviousTab => {
                    if event == NextTab {
                        self.next_tab(tabs).or(tabs.first())
                    } else {
                        self.previous_tab(tabs).or(tabs.last())
                    }
                    .map(|branch| {
                        self.position.clear();
                        self.position.push(Node::Tree(branch.get_name()));
                        self.position.push(branch.into())
                    });
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
                    self.current_branch(tabs)
                        .and_then(|branch| {
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
                                    .find_map(|branch| {
                                        self.position.last().and_then(|node| {
                                            if branch.get_name() == *node.text()
                                                && !branch.is_empty()
                                            {
                                                Some(branch)
                                            } else {
                                                None
                                            }
                                        })
                                    })
                                    .map(|current| (Node::from(current), false)),
                            }
                        })
                        .map(|(node, replace)| {
                            if replace {
                                self.position.pop();
                            }
                            self.position.push(node)
                        });
                }
                PreviousLevel => {
                    self.current_branch(tabs)
                        .and_then(|branch| match branch {
                            Branch::Args(_) => Some(()),
                            Branch::Tree(_) => None,
                        })
                        .map_or(Some(()), |_| {
                            self.position.last_mut().and_then(|node| node.dec_index())
                        })
                        .map(|_| {
                            if self.position.len() > 2 {
                                self.position.pop();
                            }
                        });
                }
                _ => (),
            }
        }
    }

    fn enter_handler<'a>(&mut self, tabs: &mut Vec<Branch<'a>>, event: crate::Event) {
        if let Some(value) = self.current_value(tabs) {
            if value.is_bool() {
                value.as_bool_mut().map(|value| *value = !*value);
            } else if let Some(text) = value.as_text_mut() {
                use crate::Event::*;
                if !self.input && event == Enter {
                    self.input = true;
                } else if self.input {
                    match event {
                        NextLevel => text.move_cursor(tui_textarea::CursorMove::Forward),
                        PreviousLevel => text.move_cursor(tui_textarea::CursorMove::Back),
                        Enter => self.input = false,
                        Cancel => todo!(),
                        Char(sym) => text.insert_char(sym),
                        Backspace => {
                            text.delete_char();
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}
