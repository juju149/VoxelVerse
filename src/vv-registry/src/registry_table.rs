use std::{collections::HashMap, marker::PhantomData};

use crate::ContentKey;

#[derive(Debug, Clone)]
pub struct RegistryTable<I, T> {
    entries: Vec<T>,
    keys: Vec<ContentKey>,
    by_key: HashMap<ContentKey, I>,
    _id: PhantomData<I>,
}

impl<I, T> Default for RegistryTable<I, T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            keys: Vec::new(),
            by_key: HashMap::new(),
            _id: PhantomData,
        }
    }
}

impl<I, T> RegistryTable<I, T>
where
    I: Copy + From<u32>,
{
    pub fn push(&mut self, key: ContentKey, value: T) -> I {
        let id = I::from(self.entries.len() as u32);
        self.entries.push(value);
        self.keys.push(key.clone());
        self.by_key.insert(key, id);
        id
    }
}

impl<I, T> RegistryTable<I, T>
where
    I: Copy + Into<usize>,
{
    pub fn get(&self, id: I) -> Option<&T> {
        self.entries.get(id.into())
    }

    pub fn key(&self, id: I) -> Option<&ContentKey> {
        self.keys.get(id.into())
    }
}

impl<I, T> RegistryTable<I, T>
where
    I: Copy,
{
    pub fn id(&self, key: &ContentKey) -> Option<I> {
        self.by_key.get(key).copied()
    }

    pub fn contains_key(&self, key: &ContentKey) -> bool {
        self.by_key.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[T] {
        &self.entries
    }

    pub fn keys(&self) -> &[ContentKey] {
        &self.keys
    }
}
