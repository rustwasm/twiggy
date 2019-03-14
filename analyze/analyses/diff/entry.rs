use std::cmp;

use serde::{self, ser::SerializeStruct};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiffEntry {
    pub name: String,
    pub delta: i64,
}

impl PartialOrd for DiffEntry {
    fn partial_cmp(&self, rhs: &DiffEntry) -> Option<cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for DiffEntry {
    fn cmp(&self, rhs: &DiffEntry) -> cmp::Ordering {
        rhs.delta
            .abs()
            .cmp(&self.delta.abs())
            .then(self.name.cmp(&rhs.name))
    }
}

impl serde::Serialize for DiffEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("DiffEntry", 2)?;
        state.serialize_field("DeltaBytes", &format!("{:+}", self.delta))?;
        state.serialize_field("Item", &self.name)?;
        state.end()
    }
}

