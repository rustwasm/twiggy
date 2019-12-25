#[derive(Debug, PartialEq, Eq)]
pub(super) struct MonosEntry {
    pub name: String,
    pub insts: Vec<(String, u32)>,
    pub size: u32,
    pub bloat: u32,
}

impl PartialOrd for MonosEntry {
    fn partial_cmp(&self, rhs: &MonosEntry) -> Option<std::cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for MonosEntry {
    fn cmp(&self, rhs: &MonosEntry) -> std::cmp::Ordering {
        rhs.bloat
            .cmp(&self.bloat)
            .then(rhs.size.cmp(&self.size))
            .then(self.insts.cmp(&rhs.insts))
            .then(self.name.cmp(&rhs.name))
    }
}
