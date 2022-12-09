use crate::{Args, Branch, Tree};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
    sync::{Arc, Mutex},
};
use tui_textarea::TextArea;

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
pub enum ValueVariant<'a> {
    Bool(bool),
    TextArea(Arc<Mutex<TextArea<'a>>>),
    Array(Vec<ValueVariant<'a>>),
    Struct(Branch<'a>),
}
impl<'a> Display for ValueVariant<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ValueVariant::Bool(value) => if *value { "True" } else { "False" }.to_string(),
                ValueVariant::TextArea(text) => text
                    .lock()
                    .unwrap()
                    .lines()
                    .get(0)
                    .unwrap_or(&String::default())
                    .clone(),
                ValueVariant::Array(arr) => format!("Count: {}", arr.len()),
                ValueVariant::Struct(_) => "->".to_string(),
            }
        )
    }
}
impl<'a> From<TextArea<'a>> for ValueVariant<'a> {
    fn from(ta: TextArea<'a>) -> Self {
        Self::TextArea(Arc::new(Mutex::new(ta)))
    }
}

#[derive(Clone)]
pub struct Value<'a>(pub(super) Type, pub(super) ValueVariant<'a>);
impl<'a> Default for Value<'a> {
    fn default() -> Self {
        Self(
            Type::None,
            ValueVariant::TextArea(Arc::new(Mutex::new(TextArea::default()))),
        )
    }
}
impl<'a> Value<'a> {
    pub fn get_type(&self) -> &Type {
        &self.0
    }

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
    pub fn as_text(&self) -> Option<Arc<Mutex<TextArea<'a>>>> {
        if let ValueVariant::TextArea(text) = &self.1 {
            Some(Arc::clone(text))
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

    pub fn parse<T: FromStr>(&self) -> Option<T> {
        self.as_text().and_then(|text| {
            text.lock()
                .unwrap()
                .lines()
                .get(0)
                .and_then(|str| str.parse::<T>().ok())
        })
    }

    pub fn check(&self) -> bool {
        if self.as_text().is_some() {
            match &self.0 {
                Type::Number(ty) => match ty {
                    NumberType::U8 => self.parse::<u8>().is_some(),
                    NumberType::I8 => self.parse::<i8>().is_some(),
                    NumberType::U16 => self.parse::<u16>().is_some(),
                    NumberType::I16 => self.parse::<i16>().is_some(),
                    NumberType::U32 => self.parse::<u32>().is_some(),
                    NumberType::I32 => self.parse::<i32>().is_some(),
                    NumberType::U64 => self.parse::<u64>().is_some(),
                    NumberType::I64 => self.parse::<i64>().is_some(),
                    NumberType::F32 => self.parse::<f32>().is_some(),
                    NumberType::F64 => self.parse::<f64>().is_some(),
                    NumberType::Usize => self.parse::<usize>().is_some(),
                    NumberType::Isize => self.parse::<isize>().is_some(),
                },
                Type::String(_) => self.parse::<String>().is_some(),
                _ => true,
            }
        } else {
            true
        }
    }

    fn setup(self) -> Self {
        if let ValueVariant::TextArea(text) = &self.1 {
            let mut text = text.lock().unwrap();
            text.set_max_histories(1);
            text.move_cursor(tui_textarea::CursorMove::End)
        }
        self
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
            TextArea::new(vec![value.to_string()]).into(),
        )
        .setup()
    }
}
impl<'a> From<i8> for Value<'a> {
    fn from(value: i8) -> Self {
        Self(
            Type::Number(NumberType::I8),
            TextArea::new(vec![value.to_string()]).into(),
        )
        .setup()
    }
}
impl<'a> From<u16> for Value<'a> {
    fn from(value: u16) -> Self {
        Self(
            Type::Number(NumberType::U16),
            TextArea::new(vec![value.to_string()]).into(),
        )
        .setup()
    }
}
impl<'a> From<i16> for Value<'a> {
    fn from(value: i16) -> Self {
        Self(
            Type::Number(NumberType::I16),
            TextArea::new(vec![value.to_string()]).into(),
        )
        .setup()
    }
}
impl<'a> From<u32> for Value<'a> {
    fn from(value: u32) -> Self {
        Self(
            Type::Number(NumberType::U32),
            TextArea::new(vec![value.to_string()]).into(),
        )
        .setup()
    }
}
impl<'a> From<i32> for Value<'a> {
    fn from(value: i32) -> Self {
        Self(
            Type::Number(NumberType::I32),
            TextArea::new(vec![value.to_string()]).into(),
        )
        .setup()
    }
}
impl<'a> From<u64> for Value<'a> {
    fn from(value: u64) -> Self {
        Self(
            Type::Number(NumberType::U64),
            TextArea::new(vec![value.to_string()]).into(),
        )
        .setup()
    }
}
impl<'a> From<i64> for Value<'a> {
    fn from(value: i64) -> Self {
        Self(
            Type::Number(NumberType::I64),
            TextArea::new(vec![value.to_string()]).into(),
        )
        .setup()
    }
}
impl<'a> From<f32> for Value<'a> {
    fn from(value: f32) -> Self {
        Self(
            Type::Number(NumberType::F32),
            TextArea::new(vec![value.to_string()]).into(),
        )
        .setup()
    }
}
impl<'a> From<f64> for Value<'a> {
    fn from(value: f64) -> Self {
        Self(
            Type::Number(NumberType::F64),
            TextArea::new(vec![value.to_string()]).into(),
        )
        .setup()
    }
}
impl<'a> From<usize> for Value<'a> {
    fn from(value: usize) -> Self {
        Self(
            Type::Number(NumberType::Usize),
            TextArea::new(vec![value.to_string()]).into(),
        )
        .setup()
    }
}
impl<'a> From<isize> for Value<'a> {
    fn from(value: isize) -> Self {
        Self(
            Type::Number(NumberType::Isize),
            TextArea::new(vec![value.to_string()]).into(),
        )
        .setup()
    }
}
impl<'a> From<String> for Value<'a> {
    fn from(value: String) -> Self {
        Self(
            Type::String(StringType::String),
            TextArea::new(vec![value]).into(),
        )
        .setup()
    }
}
impl<'a> From<&str> for Value<'a> {
    fn from(value: &str) -> Self {
        Self(
            Type::String(StringType::String),
            TextArea::new(vec![value.to_string()]).into(),
        )
        .setup()
    }
}
impl<'a> From<char> for Value<'a> {
    fn from(value: char) -> Self {
        Self(
            Type::String(StringType::Char),
            TextArea::new(vec![value.to_string()]).into(),
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
