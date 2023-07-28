use std::fmt;

use indexmap::{map::Entry, IndexMap};

use super::PrettyPrint;

#[derive(Clone)]
pub struct IdMap<T: MappedToId>(IndexMap<Box<[u8]>, T>);

impl<T: MappedToId> IdMap<T> {
    pub fn new() -> IdMap<T> {
        IdMap(IndexMap::new())
    }

    pub fn insert(&mut self, name: &[u8], value: T) -> Result<T::Id, InsertError> {
        let len = self.0.len();
        match self.0.entry(name.to_vec().into_boxed_slice()) {
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
}

pub enum InsertError {
    DuplicateName,
    NoMoreIds,
}

impl<T: MappedToId + fmt::Debug> fmt::Debug for IdMap<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Key<'a>(&'a [u8], usize);
        impl<'a> fmt::Debug for Key<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?} @ {}", PrettyPrint(self.0), self.1)
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

pub trait Id {
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
