use crate::{array::Array, state::State, widget::DrawerRef, Args, Tree};
use linked_hash_map::LinkedHashMap;
use paste::paste;

pub type Branches<'a> = LinkedHashMap<String, Branch<'a>>;

macro_rules! branch {
    {$($name:ident, $ty:ty => $list:expr )+} => {
        paste! {
            #[derive(Debug, Clone)]
            pub enum Branch<'a> {
                $([< $name:camel >] ($ty),)+
            }

            impl<'a> Branch<'a> {
                $(pub fn [<is_ $name>] (&self) -> bool {
                        matches!(self, Self:: [<$name:camel>] (_))
                    }
                    pub fn [<as_ $name>] (&self) -> Option<& $ty> {
                        match self {
                            Self:: [<$name:camel>] (v) => Some(v),
                            _ => None
                        }
                    }
                    pub fn [<as_ $name _mut>] (&mut self) -> Option<&mut $ty> {
                        match self {
                            Self:: [<$name:camel>] (v) => Some(v),
                            _ => None
                        }
                    })+

                pub fn get_list(&self) -> Vec<String> {
                    match self {$(
                        Self:: [< $name:camel >] ($name) => $list,
                    )+}
                }
                pub fn is_empty(&self) -> bool {
                    self.get_list().is_empty()
                }
            }

            impl DrawerRef for Branch<'_> {
                fn render(
                    &self,
                    area: tui::layout::Rect,
                    buf: &mut tui::buffer::Buffer,
                    state: (&State, &mut usize),
                ) {
                    match self {$(
                        Self:: [< $name:camel >] ($name) => $name .render(area, buf, state),
                    )+}
                }
            }

            $(impl<'a> From<$ty> for Branch<'a> {
                fn from($name: $ty) -> Self {
                    Self::[< $name:camel >] ($name)
                }
            })+
        }
    };
}

branch! {
    args, Args<'a> => args.get_names().into_iter().map(|span| span.content.to_string()).collect()
    tree, Tree<'a> => tree.get_branches().iter().map(|(name, _)| name.clone()).collect()
    array, Array<'a> => array.get_branches().iter().map(|(name, _)| name.clone()).collect()
}
