use std::{fmt, ops::Index};

use indexmap::{map::Entry, IndexMap};
use smartstring::alias::String;

#[derive(Clone)]
pub struct IdMap<T: MappedToId>(IndexMap<String, T>);

impl<T: MappedToId> IdMap<T> {
    pub fn new() -> IdMap<T> {
        IdMap(IndexMap::new())
    }

    pub fn insert(&mut self, name: impl Into<String>, value: T) -> Result<T::Id, InsertError> {
        let len = self.0.len();
        match self.0.entry(name.into()) {
            Entry::Occupied(_) => Err(InsertError::DuplicateName),
            Entry::Vacant(entry) => {
                if len < T::Id::max_value() {
                    entry.insert(value);
                    Ok(T::Id::from_usize(len))
                } else {
                    Err(InsertError::NoMoreIds)
                }
            }
        }
    }

    pub fn get(&self, index: T::Id) -> Option<&T> {
        self.0.get_index(index.to_usize()).map(|(_, value)| value)
    }

    pub fn get_full(&self, index: T::Id) -> Option<(&str, &T)> {
        self.0
            .get_index(index.to_usize())
            .map(|(key, value)| (&**key, value))
    }

    pub fn get_full_by_name(&self, name: &str) -> Option<(T::Id, &T)> {
        self.0
            .get_full(name)
            .map(|(id, _, value)| (T::Id::from_usize(id), value))
    }

    pub fn iter(&self) -> impl Iterator<Item = (T::Id, &str, &T)> {
        self.0
            .iter()
            .enumerate()
            .map(|(i, (key, value))| (T::Id::from_usize(i), &**key, value))
    }
}

impl<T: MappedToId> Index<T::Id> for IdMap<T> {
    type Output = T;

    fn index(&self, index: T::Id) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("Map does not contain id {}", index.to_usize()))
    }
}

impl<T: CreateInstanceWithId> IdMap<T> {
    pub fn instance_of(&self, id: T::Id) -> Option<T::Instance> {
        self.get(id).map(|class| class.create_instance(id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertError {
    DuplicateName,
    NoMoreIds,
}

impl<T: MappedToId + fmt::Debug> fmt::Debug for IdMap<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Key<'a>(&'a str, usize);
        impl<'a> fmt::Debug for Key<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?} @ {}", self.0, self.1)
            }
        }

        f.debug_map()
            .entries(
                self.0
                    .iter()
                    .enumerate()
                    .map(|(id, (k, v))| (Key(k, id), v)),
            )
            .finish()
    }
}

pub trait Id: Copy {
    fn to_usize(self) -> usize;

    fn from_usize(id: usize) -> Self;

    fn max_value() -> usize;
}

macro_rules! id_impl {
    ($( $ty:ty ),+) => {
        $(
            impl Id for $ty {
                fn to_usize(self) -> usize {
                    self as usize
                }

                fn from_usize(id: usize) -> Self {
                    id as Self
                }

                fn max_value() -> usize {
                    Self::MAX as usize
                }
            }
        )+
    };
}

id_impl!(u8, u16, u32, usize);

pub trait MappedToId: Sized {
    type Id: Id;

    fn create_map() -> IdMap<Self> {
        IdMap::new()
    }
}

pub trait CreateInstanceWithId: MappedToId {
    type Instance;

    fn create_instance(&self, id: Self::Id) -> Self::Instance;
}
