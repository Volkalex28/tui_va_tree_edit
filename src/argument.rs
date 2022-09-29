use std::fmt::{Debug, Display};

use linked_hash_map::LinkedHashMap;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Widget},
};
use tui_textarea::TextArea;

use crate::{
    state::State,
    tree::{Branch, Tree},
    widget::DrawerRef,
};

#[derive(Debug, Clone)]
pub enum NumberType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    Usize,
    Isize,
}
impl Display for NumberType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NumberType::U8 => "u8",
                NumberType::I8 => "i8",
                NumberType::U16 => "u16",
                NumberType::I16 => "i16",
                NumberType::U32 => "u32",
                NumberType::I32 => "i32",
                NumberType::U64 => "u32",
                NumberType::I64 => "i32",
                NumberType::F32 => "f32",
                NumberType::F64 => "f64",
                NumberType::Usize => "usize",
                NumberType::Isize => "isize",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub enum StringType {
    Char,
    String,
}
impl Display for StringType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                StringType::Char => "Symbol",
                StringType::String => "String",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    None,
    Bool,
    Number(NumberType),
    String(StringType),
    Array(Box<Type>),
    Struct,
}
impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::None => "None".to_string(),
                Type::Bool => "Bool".to_string(),
                Type::Number(ty) => ty.to_string(),
                Type::String(ty) => ty.to_string(),
                Type::Array(ty) => format!("Array<{}>", ty.as_ref()),
                Type::Struct => "Struct".to_string(),
            }
        )
    }
}

#[derive(Clone)]
enum ValueVariant<'a> {
    // String(Span<'a>),
    Bool(bool),
    TextArea(TextArea<'a>),
    Array(Vec<ValueVariant<'a>>),
    Struct(Branch<'a>),
}
impl<'a> Display for ValueVariant<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                // ValueVariant::String(str) => str.content.to_string(),
                ValueVariant::Bool(value) => if *value { "True" } else { "False" }.to_string(),
                ValueVariant::TextArea(text) =>
                    text.lines().get(0).unwrap_or(&String::default()).clone(),
                ValueVariant::Array(arr) => format!("Count: {}", arr.len()),
                ValueVariant::Struct(_) => "->".to_string(),
            }
        )
    }
}

#[derive(Clone)]
pub struct Value<'a>(Type, ValueVariant<'a>);
impl<'a> Default for Value<'a> {
    fn default() -> Self {
        Self(Type::None, ValueVariant::TextArea(TextArea::default()))
    }
}
impl<'a> Value<'a> {
    pub fn is_none(&self) -> bool {
        matches!(self.0, Type::None)
    }
    pub fn is_bool(&self) -> bool {
        matches!(self.0, Type::Bool)
    }
    pub fn is_number(&self) -> bool {
        matches!(self.0, Type::Number(_))
    }
    pub fn is_string(&self) -> bool {
        matches!(self.0, Type::String(_))
    }
    pub fn is_array(&self) -> bool {
        matches!(self.0, Type::Array(_))
    }
    pub fn is_struct(&self) -> bool {
        matches!(self.0, Type::Struct)
    }

    pub fn as_bool(&self) -> Option<&bool> {
        if let ValueVariant::Bool(value) = &self.1 {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_bool_mut(&mut self) -> Option<&mut bool> {
        if let ValueVariant::Bool(value) = &mut self.1 {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_text(&self) -> Option<&TextArea<'a>> {
        if let ValueVariant::TextArea(text) = &self.1 {
            Some(text)
        } else {
            None
        }
    }
    pub fn as_text_mut(&mut self) -> Option<&mut TextArea<'a>> {
        if let ValueVariant::TextArea(text) = &mut self.1 {
            Some(text)
        } else {
            None
        }
    }
    pub fn as_struct(&self) -> Option<&Branch<'a>> {
        if let ValueVariant::Struct(str) = &self.1 {
            Some(str)
        } else {
            None
        }
    }
    pub fn as_struct_mut(&mut self) -> Option<&mut Branch<'a>> {
        if let ValueVariant::Struct(str) = &mut self.1 {
            Some(str)
        } else {
            None
        }
    }

    pub fn into_array(mut self) -> Self {
        self.0 = Type::Array(Box::new(self.0));
        self.1 = ValueVariant::Array(vec![self.1]);
        self
    }
    pub fn into_clear_array(mut self) -> Self {
        self.0 = Type::Array(Box::new(self.0));
        self.1 = ValueVariant::Array(vec![]);
        self
    }

    fn setup(mut self) -> Self {
        if let ValueVariant::TextArea(text) = &mut self.1 {
            text.set_max_histories(0);
            text.move_cursor(tui_textarea::CursorMove::End)
        }
        self
    }
}

#[derive(Clone)]
pub struct Args<'a> {
    name: String,
    names: LinkedHashMap<String, Vec<usize>>,
    columns: LinkedHashMap<String, Vec<usize>>,
    values: Vec<Value<'a>>,
}
impl<'a> Args<'a> {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            names: Default::default(),
            columns: Default::default(),
            values: Vec::default(),
        }
    }

    pub fn names<T>(mut self, names: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<String>,
    {
        self.names = names
            .into_iter()
            .map(|name| (name.into(), Vec::default()))
            .collect();
        self
    }
    pub fn columns<T>(mut self, columns: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<String>,
    {
        self.columns = columns
            .into_iter()
            .map(|column| (column.into(), Vec::default()))
            .collect();
        self
    }
    pub fn value<T: Into<String>, V: Into<Value<'a>>>(
        mut self,
        name: T,
        column: T,
        value: V,
    ) -> Self {
        let insert = |name: String, map: &mut LinkedHashMap<String, Vec<usize>>| {
            if let Some(vec) =
                map.iter_mut().find_map(
                    |(inner_name, vec)| {
                        if &name == inner_name {
                            Some(vec)
                        } else {
                            None
                        }
                    },
                )
            {
                vec.push(self.values.len())
            } else {
                map.insert(name, vec![self.values.len()]);
            }
        };
        insert(name.into(), &mut self.names);
        insert(column.into(), &mut self.columns);
        self.values.push(value.into());
        self
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_names(&self) -> Vec<Span<'a>> {
        self.names
            .iter()
            .map(|(name, _)| name.clone().into())
            .collect()
    }
    pub fn get_columns(&self) -> Vec<Span<'a>> {
        self.columns
            .iter()
            .map(|(name, _)| name.clone().into())
            .collect()
    }
    pub fn get_value<T: Into<String>>(&self, name: T, column: T) -> Option<&Value<'a>> {
        if let Some(names) = self.names.get(&name.into()) {
            if let Some(columns) = self.columns.get(&column.into()) {
                return names
                    .iter()
                    .find(|&index| columns.contains(index))
                    .map_or(None, |&index| self.values.get(index));
            }
        }
        None
    }
    pub fn get_value_mut<T: Into<String>>(&mut self, name: T, column: T) -> Option<&mut Value<'a>> {
        if let Some(names) = self.names.get(&name.into()) {
            if let Some(columns) = self.columns.get(&column.into()) {
                return names
                    .iter()
                    .find(|&index| columns.contains(index))
                    .map_or(None, |&index| self.values.get_mut(index));
            }
        }
        None
    }
    pub fn get_value_by_indexes(&self, name: usize, column: usize) -> Option<&Value<'a>> {
        self.names
            .iter()
            .skip(name)
            .next()
            .map(|(_, names)| names)
            .zip(
                self.columns
                    .iter()
                    .skip(column)
                    .next()
                    .map(|(_, columns)| columns),
            )
            .and_then(|(names, columns)| {
                names
                    .iter()
                    .find(|&index| columns.contains(index))
                    .map_or(None, |&index| self.values.get(index))
            })
    }
    pub fn get_value_by_index(&self, index: usize) -> Option<&Value<'a>> {
        self.values.get(index)
    }
}
impl<'a> DrawerRef<'a> for Args<'a> {
    fn render(
        &'a self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: (&State, &mut usize),
    ) {
        enum ToRender<'b> {
            Text(TextArea<'b>),
            Paragraph(Paragraph<'b>),
        }
        enum Text<'b> {
            Text(&'b TextArea<'b>),
            Span(Span<'b>),
        }

        if *state.1 > state.0.position.len() {
            return;
        }

        let current = {
            state.0.node(*state.1).and_then(|current| {
                current.args().and_then(|(current, col)| {
                    self.names
                        .iter()
                        .enumerate()
                        .find_map(|(number, (text, _))| {
                            if *text == *current {
                                Some((number + 1, col + 1))
                            } else {
                                None
                            }
                        })
                })
            })
        };
        let chunks = {
            let values = {
                self.columns
                    .iter()
                    .map(|(column, _)| {
                        [("".to_string(), Text::Span(Span::from(column.clone())))]
                            .into_iter()
                            .chain(self.names.iter().map(|(name, _)| {
                                self.get_value(name, column).map_or(
                                    ("".to_string(), Text::Span(Span::from(""))),
                                    |value| {
                                        (
                                            value.0.to_string(),
                                            match &value.1 {
                                                ValueVariant::TextArea(text) => {
                                                    Text::Text::<'a>(text)
                                                }
                                                value => Text::Span(Span::from(value.to_string())),
                                            },
                                        )
                                    },
                                )
                            }))
                            .collect()
                    })
                    .collect::<Vec<Vec<(String, Text<'a>)>>>()
            };

            let names = {
                ["".to_string()]
                    .into_iter()
                    .chain(self.names.iter().map(|(name, _)| name.clone()))
                    .map(|name| ("".to_string(), Text::Span(Span::from(name))))
                    .collect::<Vec<(String, Text<'a>)>>()
            };

            let widths = {
                [&names]
            .into_iter()
            .chain(values.iter())
            .map(|values| {
                    values.iter().fold(0usize, |max, (name, value)| {
                        max.max(name.len() + 1).max(match value {
                            Text::Text(text) => text.lines().first().map_or(0, |str| str.len()),
                            Text::Span(span) => span.width()
                        })
                    })
                } + 3)
            .collect::<Vec<usize>>()
            };

            let chunks = {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(
                            widths.iter().fold(1usize, |res, width| res + *width) as u16
                        ),
                        Constraint::Min(2),
                    ])
                    .split(area)
            };

            let inner_area = {
                let block = Block::default()
                    // .border_type(tui::widgets::BorderType::Thick)
                    .style(state.0.style)
                    .borders(Borders::RIGHT);
                let inner_area = block.inner(chunks[0]);
                tui::widgets::Widget::render(block, chunks[0], buf);
                inner_area
            };

            let inner_chunks = {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        widths
                            .iter()
                            .map(|width| Constraint::Length(*width as u16))
                            .chain([Constraint::Min(2)].into_iter())
                            .collect::<Vec<Constraint>>(),
                    )
                    .split(inner_area)
                    .into_iter()
                    .map(|rect| {
                        Layout::default()
                            .direction(Direction::Vertical)
                            .constraints(
                                names
                                    .iter()
                                    .enumerate()
                                    .map(|(index, _)| {
                                        Constraint::Length(if index > 0 { 2 } else { 1 })
                                    })
                                    .chain([Constraint::Min(2)].into_iter())
                                    .collect::<Vec<Constraint>>(),
                            )
                            .split(rect)
                    })
                    .collect::<Vec<Vec<Rect>>>()
            };

            [names]
                .into_iter()
                .chain(values.into_iter())
                .enumerate()
                .map(|(col_index, values)| {
                    values
                        .into_iter()
                        .enumerate()
                        .map(|(index, (block, span))| {
                            let highlight_style = current.and_then(|(row, col)| {
                                if (index == row && col_index == col
                                    || (index == 0 && col_index == col
                                        || index == row && col_index == 0)
                                        && *state.1 + 1 == state.0.position.len())
                                    && !state.0.input
                                {
                                    Some(state.0.highlight_style)
                                } else {
                                    None
                                }
                            });
                            match span {
                                Text::Text(text) => {
                                    let cursor_style = current.and_then(|(row, col)| {
                                        if row == index && col == col_index && state.0.input {
                                            Some(())
                                        } else {
                                            None
                                        }
                                    });
                                    let mut text = text.clone();
                                    text.set_block(
                                        Block::default()
                                            .title(block)
                                            .borders(Borders::TOP)
                                            .border_style(
                                                Style::default().add_modifier(Modifier::ITALIC),
                                            ),
                                    );
                                    highlight_style
                                        .map_or(text.set_style(state.0.style), |style| {
                                            text.set_style(style)
                                        });
                                    text.set_cursor_style(if cursor_style.is_none() {
                                        text.style()
                                    } else {
                                        Style::default().add_modifier(Modifier::REVERSED)
                                    });
                                    text.set_alignment(if cursor_style.is_some() {
                                        Alignment::Left
                                    } else {
                                        Alignment::Center
                                    });
                                    ToRender::Text(text)
                                }
                                Text::Span(span) => {
                                    let spans = Spans::from(
                                        if let Some(style) = current.and_then(|(row, col)| {
                                            if (/*(row == index && col_index == 0 && !state.0.input)
                                            ||*/index == 0 && col_index == col && !state.0.input
                                                || col_index == 0 && index == row && !state.0.input
                                                || (row == index
                                                    && col == col_index
                                                    && state.0.input))
                                                && *state.1 + 1 == state.0.position.len()
                                            {
                                                Some(state.0.highlight_style)
                                            } else {
                                                None
                                            }
                                        }) {
                                            Span::styled(span.content, style)
                                        } else if index == 0 || col_index == 0 {
                                            Span::styled(
                                                span.content,
                                                // state.0.style.patch(
                                                Style::default().add_modifier(Modifier::DIM),
                                                // ),
                                            )
                                        } else {
                                            span
                                        },
                                    );

                                    let paragraph = if index > 0 {
                                        Paragraph::new(spans).block(
                                            Block::default()
                                                .title(block)
                                                .borders(if col_index > 0 {
                                                    Borders::TOP
                                                } else {
                                                    Borders::TOP | Borders::RIGHT
                                                })
                                                .border_style(
                                                    Style::default().add_modifier(Modifier::ITALIC),
                                                ),
                                        )
                                    } else {
                                        let paragraph = Paragraph::new(spans);
                                        if col_index == 0 {
                                            paragraph
                                                .block(Block::default().borders(Borders::RIGHT))
                                        } else {
                                            paragraph
                                        }
                                    }
                                    .alignment(tui::layout::Alignment::Center);

                                    ToRender::Paragraph(if let Some(style) = highlight_style {
                                        paragraph.style(style)
                                    } else {
                                        paragraph.style(state.0.style)
                                    })
                                    // };
                                }
                            }
                        })
                        .collect::<Vec<ToRender<'a>>>()
                })
                .zip(inner_chunks.into_iter())
                .map(|(paragraphes, chunks)| paragraphes.into_iter().zip(chunks.into_iter()))
                .for_each(|paragraphes| {
                    paragraphes.for_each(|(paragraph, area)| match paragraph {
                        ToRender::Text(text) => {
                            text.widget().render(area, buf);
                            // buf.set_style(area, style);
                        }
                        ToRender::Paragraph(paragraph) => paragraph.render(area, buf),
                    })
                });

            chunks
        };

        if let Some(branch) = current.and_then(|(name, col)| {
            self.names
                .iter()
                .skip(name - 1)
                .next()
                .and_then(|(name, _)| {
                    self.columns
                        .iter()
                        .skip(col - 1)
                        .next()
                        .and_then(|(column, _)| {
                            self.get_value(name, column).and_then(|value| {
                                if let ValueVariant::Struct(branch) = &value.1 {
                                    Some(branch)
                                } else {
                                    None
                                }
                            })
                        })
                })
        }) {
            *state.1 += 1;
            branch.render(chunks[1], buf, state)
        }
    }
}

impl<'a> From<bool> for Value<'a> {
    fn from(value: bool) -> Self {
        Self(Type::Bool, ValueVariant::Bool(value))
    }
}
impl<'a> From<u8> for Value<'a> {
    fn from(value: u8) -> Self {
        Self(
            Type::Number(NumberType::U8),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<i8> for Value<'a> {
    fn from(value: i8) -> Self {
        Self(
            Type::Number(NumberType::I8),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<u16> for Value<'a> {
    fn from(value: u16) -> Self {
        Self(
            Type::Number(NumberType::U16),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<i16> for Value<'a> {
    fn from(value: i16) -> Self {
        Self(
            Type::Number(NumberType::I16),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<u32> for Value<'a> {
    fn from(value: u32) -> Self {
        Self(
            Type::Number(NumberType::U32),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<i32> for Value<'a> {
    fn from(value: i32) -> Self {
        Self(
            Type::Number(NumberType::I32),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<u64> for Value<'a> {
    fn from(value: u64) -> Self {
        Self(
            Type::Number(NumberType::U64),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<i64> for Value<'a> {
    fn from(value: i64) -> Self {
        Self(
            Type::Number(NumberType::I64),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<f32> for Value<'a> {
    fn from(value: f32) -> Self {
        Self(
            Type::Number(NumberType::F32),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<f64> for Value<'a> {
    fn from(value: f64) -> Self {
        Self(
            Type::Number(NumberType::F64),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<usize> for Value<'a> {
    fn from(value: usize) -> Self {
        Self(
            Type::Number(NumberType::Usize),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<isize> for Value<'a> {
    fn from(value: isize) -> Self {
        Self(
            Type::Number(NumberType::Isize),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<String> for Value<'a> {
    fn from(value: String) -> Self {
        Self(
            Type::String(StringType::String),
            ValueVariant::TextArea(TextArea::new(vec![value])),
        )
        .setup()
    }
}
impl<'a> From<&str> for Value<'a> {
    fn from(value: &str) -> Self {
        Self(
            Type::String(StringType::String),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<char> for Value<'a> {
    fn from(value: char) -> Self {
        Self(
            Type::String(StringType::Char),
            ValueVariant::TextArea(TextArea::new(vec![value.to_string()])),
        )
        .setup()
    }
}
impl<'a> From<Tree<'a>> for Value<'a> {
    fn from(value: Tree<'a>) -> Self {
        Self(Type::Struct, ValueVariant::Struct(value.into()))
    }
}
impl<'a> From<Args<'a>> for Value<'a> {
    fn from(value: Args<'a>) -> Self {
        Self(Type::Struct, ValueVariant::Struct(value.into()))
    }
}

#[cfg(test)]
mod tests {
    use tui::text::Span;

    use crate::argument::{Args, Value};

    #[test]
    fn it_works() {
        let args = Args::new("")
            .names(["Name 1", "Name 2"])
            .columns(["Column 1", "Column 2"])
            .value("Name 1", "Column 2", false);

        assert_eq!(
            matches!(
                args.get_value("Name 1", "Column 2"),
                Some(Value(crate::argument::Type::Bool, _))
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
