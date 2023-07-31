use std::ops::{Deref, DerefMut};

pub struct DetectChanges<'a, T> {
    val: &'a mut T,
    changed: &'a mut bool,
}

impl<'a, T> DetectChanges<'a, T> {
    pub fn new(val: &'a mut T, changed: &'a mut bool) -> Self {
        Self { val, changed }
    }
}

impl<'a, T> Deref for DetectChanges<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.val
    }
}

impl<'a, T> DerefMut for DetectChanges<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        *self.changed = true;
        self.val
    }
}
