use crate::{array::Array, Args, Branch, Tree};
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

pub enum ValueVariant<'a> {
    Bool(bool),
    TextArea(Arc<Mutex<TextArea<'a>>>),
    Struct(Branch<'a>),
}
impl Debug for ValueVariant<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(arg0) => f.debug_tuple("Bool").field(arg0).finish(),
            Self::TextArea(arg0) => f
                .debug_tuple("TextArea")
                .field(&arg0.lock().unwrap().lines()[0])
                .finish(),
            Self::Struct(arg0) => f.debug_tuple("Struct").field(arg0).finish(),
        }
    }
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
                ValueVariant::Struct(arr @ Branch::Array(_)) =>
                    format!("Count: {}", arr.get_list().len()),
                ValueVariant::Struct(_) => "->".to_string(),
            }
        )
    }
}
impl Clone for ValueVariant<'_> {
    fn clone(&self) -> Self {
        match self {
            Self::Bool(arg0) => Self::Bool(arg0.clone()),
            Self::TextArea(arg0) => {
                Self::TextArea(Arc::new(Mutex::new(arg0.lock().unwrap().clone())))
            }
            Self::Struct(arg0) => Self::Struct(arg0.clone()),
        }
    }
}
impl<'a> From<TextArea<'a>> for ValueVariant<'a> {
    fn from(ta: TextArea<'a>) -> Self {
        Self::TextArea(Arc::new(Mutex::new(ta)))
    }
}

#[derive(Debug, Clone)]
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
    #[allow(unused)]
    pub fn as_array(&self) -> Option<&Array<'a>> {
        if let ValueVariant::Struct(Branch::Array(tree)) = &self.1 {
            Some(tree)
        } else {
            None
        }
    }
    pub(crate) fn as_array_mut(&mut self) -> Option<&mut Array<'a>> {
        if let ValueVariant::Struct(Branch::Array(tree)) = &mut self.1 {
            Some(tree)
        } else {
            None
        }
    }

    pub fn into_array(mut self) -> Self {
        self.1 = ValueVariant::Struct(Array::new(self.clone().into()).into());
        self.0 = Type::Array(Box::new(self.0));
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

macro_rules! impl_get_type {
    ($($ty:ty $(,)?)+ => $ret:expr) => {
        $(
            impl GetType for $ty {
                fn get() -> Type {
                    $ret
                }
            }
        )+
    };
}
macro_rules! impl_from_for_value {
    ($($ty:ty $(,)?)+ => $var:expr; $ident:ident) => {
        $(impl<'a> From<$ty> for Value<'a> {
            fn from($ident: $ty) -> Self {
                Self(
                    <$ty as GetType>::get(),
                    $var,
                )
                .setup()
            }
        })+
    };
}

trait GetType {
    fn get() -> Type;
}
impl_get_type!(bool => Type::Bool);
impl_get_type!(u8 => Type::Number(NumberType::U8));
impl_get_type!(i8 => Type::Number(NumberType::I8));
impl_get_type!(u16 => Type::Number(NumberType::U16));
impl_get_type!(i16 => Type::Number(NumberType::I16));
impl_get_type!(u32 => Type::Number(NumberType::U32));
impl_get_type!(i32 => Type::Number(NumberType::I32));
impl_get_type!(u64 => Type::Number(NumberType::U64));
impl_get_type!(i64 => Type::Number(NumberType::I64));
impl_get_type!(f32 => Type::Number(NumberType::F32));
impl_get_type!(f64 => Type::Number(NumberType::F64));
impl_get_type!(usize => Type::Number(NumberType::Usize));
impl_get_type!(isize => Type::Number(NumberType::Isize));
impl_get_type!(char => Type::String(StringType::Char));
impl_get_type!(&str, String => Type::String(StringType::String));
impl_get_type!(Tree<'_>, Args<'_> => Type::Struct);

impl_from_for_value!(bool => ValueVariant::Bool(v); v);
impl_from_for_value!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64, usize, isize, String, &str, char
        => TextArea::new(vec![v.to_string()]).into(); v);
impl_from_for_value!(Tree<'a>, Args<'a>, => ValueVariant::Struct(v.into()); v);

impl<T: GetType + Default> From<Vec<T>> for Value<'_>
where
    Self: From<T>,
{
    fn from(value: Vec<T>) -> Self {
        Self(
            Type::Array(<T as GetType>::get().into()),
            ValueVariant::Struct(
                value
                    .into_iter()
                    .fold(
                        Array::new(Self::from(T::default()).into()),
                        |mut tree, v| {
                            tree.insert(None, v);
                            tree
                        },
                    )
                    .into(),
            ),
        )
    }
}
impl<T: GetType + Default, const S: usize> From<[T; S]> for Value<'_>
where
    Self: From<T>,
{
    fn from(value: [T; S]) -> Self {
        value.into_iter().collect::<Vec<_>>().into()
    }
}
