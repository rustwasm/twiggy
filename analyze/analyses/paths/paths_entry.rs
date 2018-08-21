use std::cmp;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PathsEntry {
    pub name: String,
    pub size: u32,
    pub children: Vec<PathsEntry>,
}

impl PathsEntry {
    pub fn _count(&self) -> u32 {
        1 + self.children.iter().map(|c| c._count()).sum::<u32>()
    }
}

impl PartialOrd for PathsEntry {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for PathsEntry {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        rhs.size.cmp(&self.size).then(self.name.cmp(&rhs.name))
    }
}
