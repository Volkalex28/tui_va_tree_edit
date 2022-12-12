use crate::{state::State, widget::DrawerRef};
use std::collections::HashMap;
use tui::text::Span;

use value::Value;
mod drawer;
pub mod value;

#[derive(Debug, Default, Clone)]
pub struct Args<'a> {
    names: Vec<String>,
    columns: Vec<String>,
    values: HashMap<(usize, usize), Value<'a>>,
}
impl<'a> Args<'a> {
    pub fn names<T: IntoIterator>(mut self, names: T) -> Self
    where
        T::Item: ToString,
    {
        self.names = names.into_iter().map(|name| name.to_string()).collect();
        self
    }
    pub fn columns<T: IntoIterator>(mut self, columns: T) -> Self
    where
        T::Item: ToString,
    {
        self.columns = columns
            .into_iter()
            .map(|column| column.to_string())
            .collect();
        self
    }
    pub fn value<N: ToString, C: ToString, V: Into<Value<'a>>>(
        mut self,
        name: N,
        column: C,
        value: V,
    ) -> Self {
        self.positions(&name.to_string(), &column.to_string())
            .map(|indexes| self.values.insert(indexes, value.into()));
        self
    }

    pub fn get_names_raw(&self) -> &Vec<String> {
        &self.names
    }
    pub fn get_names(&self) -> Vec<Span<'a>> {
        self.names.iter().map(|name| name.clone().into()).collect()
    }
    pub fn get_columns_raw(&self) -> &Vec<String> {
        &self.columns
    }
    pub fn get_columns(&self) -> Vec<Span<'a>> {
        self.columns.iter().map(|col| col.clone().into()).collect()
    }
    pub fn get_value(&self, name: impl ToString, column: impl ToString) -> Option<&Value<'a>> {
        self.positions(&name.to_string(), &column.to_string())
            .and_then(|indexes| self.values.get(&indexes))
    }
    pub fn get_value_mut(
        &mut self,
        name: impl ToString,
        column: impl ToString,
    ) -> Option<&mut Value<'a>> {
        self.positions(&name.to_string(), &column.to_string())
            .and_then(|indexes| self.values.get_mut(&indexes))
    }
    pub fn get_value_by_indexes(&self, name: usize, column: usize) -> Option<&Value<'a>> {
        self.values.get(&(name, column))
    }
    pub fn get_value_by_indexes_mut(
        &mut self,
        name: usize,
        column: usize,
    ) -> Option<&mut Value<'a>> {
        self.values.get_mut(&(name, column))
    }
    pub fn get_value_by_cindex(&self, name: impl ToString, column: usize) -> Option<&Value<'a>> {
        Self::position(self.names.iter(), &name.to_string())
            .and_then(|index| self.values.get(&(index, column)))
    }
    pub fn get_value_by_cindex_mut(
        &mut self,
        name: impl ToString,
        column: usize,
    ) -> Option<&mut Value<'a>> {
        Self::position(self.names.iter(), &name.to_string())
            .and_then(|index| self.values.get_mut(&(index, column)))
    }

    fn position<C: Iterator, P>(mut container: C, value: P) -> Option<usize>
    where
        C::Item: PartialEq<P>,
    {
        container.position(|item| item == value)
    }
    fn positions(&self, name: &str, column: &str) -> Option<(usize, usize)> {
        Self::position(self.names.iter(), name).zip(Self::position(self.columns.iter(), column))
    }
}
impl DrawerRef for Args<'_> {
    fn render(
        &self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: (&State, &mut usize),
    ) {
        drawer::Drawer(self).render(area, buf, state)
    }
}

#[cfg(test)]
mod tests {
    use crate::Args;
    use tui::text::Span;

    #[test]
    fn it_works() {
        let args = Args::default()
            .names(["Name 1", "Name 2"])
            .columns(["Column 1", "Column 2"])
            .value("Name 1", "Column 2", false);

        assert_eq!(
            matches!(
                args.get_value("Name 1", "Column 2"),
                Some(value) if value.is_bool()
            ),
            true
        );
        assert_eq!(matches!(args.get_value("Name 1", "Column 1"), None), true);
        assert_eq!(
            args.get_names(),
            vec![Span::from("Name 1"), Span::from("Name 2")]
        );
        assert_eq!(
            args.get_columns(),
            vec![Span::from("Column 1"), Span::from("Column 2")]
        );
    }
}
