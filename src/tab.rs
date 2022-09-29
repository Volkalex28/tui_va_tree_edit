use crate::{
    state::State,
    tree::{Branch, Tree},
    widget::DrawerRef,
};

#[derive(Clone)]
pub struct Tab<'a>(Branch<'a>);
impl<'a> Default for Tab<'a> {
    fn default() -> Self {
        Self(Tree::default())
    }
}
impl<'a> Tab<'a> {
    pub fn new<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        Self(Tree::new(name))
    }

    pub fn get_name(&self) -> String {
        self.0.get_name()
    }
    pub fn get_tree(&self) -> &Tree<'a> {
        &self.0
    }

    pub fn branch(mut self, branch: impl Into<Branch<'a>>) -> Self {
        self.0 = self.0.branch(branch);
        self
    }
}

impl<'a> DrawerRef<'a> for Tab<'a> {
    fn render(
        &self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: (&State, &mut usize),
    ) {
        self.0.render(area, buf, state);
    }
}
