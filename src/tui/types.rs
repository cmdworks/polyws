#[derive(Clone, Copy, PartialEq)]
pub(super) enum Tab {
    Dashboard = 0,
    Projects = 1,
    Graph = 2,
    Snapshots = 3,
    Sync = 4,
    Logs = 5,
}

impl Tab {
    pub(super) fn titles() -> &'static [&'static str] {
        &[
            "1:Dash", "2:Proj", "3:Graph", "4:Snap", "5:Sync", "6:Doctor",
        ]
    }

    pub(super) fn from_usize(i: usize) -> Self {
        match i {
            0 => Tab::Dashboard,
            1 => Tab::Projects,
            2 => Tab::Graph,
            3 => Tab::Snapshots,
            4 => Tab::Sync,
            _ => Tab::Logs,
        }
    }
}

#[derive(Clone)]
pub(super) struct AddForm {
    pub(super) fields: [String; 6], // name, path, url, branch, sync_url, depends_on
    pub(super) focused: usize,      // which field
}

impl Default for AddForm {
    fn default() -> Self {
        let mut fields: [String; 6] = Default::default();
        fields[3] = "main".to_string();
        Self { fields, focused: 0 }
    }
}

pub(super) const FIELD_LABELS: [&str; 6] = [
    "Name",
    "Path (optional)",
    "URL",
    "Branch",
    "Sync URL",
    "Depends On",
];

pub(super) const FIELD_HINTS: [&str; 6] = [
    "required",
    "nested/path (optional)",
    "git@host:org/repo.git",
    "main",
    "optional mirror url",
    "comma-separated project names",
];
