use crate::{Args, Tree, Value};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Array<'a> {
    pub(crate) def: Box<Value<'a>>,
    pub tree: Tree<'a>,
}
impl<'a> Array<'a> {
    pub(crate) fn new(def: Box<Value<'a>>) -> Self {
        Self {
            def,
            tree: Default::default(),
        }
    }

    pub fn is_wrapped(&self) -> bool {
        self.def.is_struct()
    }

    #[inline]
    pub(crate) fn insert_default(&mut self, index: Option<usize>) {
        self.insert(index, (*self.def).clone())
    }

    pub(crate) fn insert(&mut self, index: Option<usize>, value: impl Into<Value<'a>>) {
        let len = self.branches.len();
        let value: Value = value.into();
        let value = if let Some(b) = value.as_struct() {
            b.clone()
        } else {
            Args::default()
                .names([""])
                .columns(["Value"])
                .value("", "Value", value)
                .into()
        };
        if index.filter(|index| *index < len).is_some() {
            self.tree.branches.insert("".into(), value);
            self.update();
        } else {
            self.tree.branches.insert(len.to_string(), value);
        }
    }

    pub(crate) fn remove(&mut self, index: &String) -> Option<String> {
        let not_last = !self.branches.back().filter(|&(s, _)| s == index).is_some();
        self.tree.branches.remove(index);
        self.update();
        not_last
            .then_some(index)
            .or_else(|| self.branches.back().map(|(s, _)| s))
            .cloned()
    }

    fn update(&mut self) {
        self.tree.branches = self
            .branches
            .iter()
            .map(|(_, v)| v.clone())
            .enumerate()
            .map(|(i, v)| (i.to_string(), v))
            .collect()
    }
}

impl<'a> Deref for Array<'a> {
    type Target = Tree<'a>;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}
