use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub url: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_interval: Option<u64>,
}

fn default_branch() -> String {
    "main".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmConfig {
    pub host: String,
    pub user: String,
    pub path: String,
    #[serde(default = "default_sync")]
    pub sync: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

fn default_sync() -> String {
    "rsync".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_interval_minutes: Option<u64>,
    pub projects: Vec<Project>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vm: Option<VmConfig>,
}

impl WorkspaceConfig {
    pub fn load() -> Result<Self> {
        // Accept both .polyws and .poly
        let content = fs::read_to_string(".polyws")
            .or_else(|_| fs::read_to_string(".poly"))
            .context("No .polyws or .poly found. Are you in a workspace directory?")?;
        serde_json::from_str(&content).context("Failed to parse workspace config")
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        // Write to whichever file already exists, defaulting to .polyws
        let path = if std::path::Path::new(".poly").exists()
            && !std::path::Path::new(".polyws").exists()
        {
            ".poly"
        } else {
            ".polyws"
        };
        fs::write(path, content)?;
        Ok(())
    }

    pub fn find_project(&self, name: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.name == name)
    }

    /// Returns projects in topological order (dependencies before their dependents).
    pub fn topological_order(&self) -> Result<Vec<&Project>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();

        for project in &self.projects {
            self.topo_visit(project, &mut visited, &mut in_stack, &mut result)?;
        }
        Ok(result)
    }

    fn topo_visit<'a>(
        &'a self,
        project: &'a Project,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
        result: &mut Vec<&'a Project>,
    ) -> Result<()> {
        if visited.contains(&project.name) {
            return Ok(());
        }
        if in_stack.contains(&project.name) {
            anyhow::bail!("Cyclic dependency detected involving '{}'", project.name);
        }
        in_stack.insert(project.name.clone());

        if let Some(deps) = &project.depends_on {
            for dep_name in deps {
                if let Some(dep) = self.find_project(dep_name) {
                    self.topo_visit(dep, visited, in_stack, result)?;
                }
            }
        }

        in_stack.remove(&project.name);
        visited.insert(project.name.clone());
        result.push(project);
        Ok(())
    }

    /// Groups projects into parallel execution levels respecting dependencies.
    /// Projects in the same level may run concurrently; each level waits for the previous.
    pub fn execution_levels(&self) -> Result<Vec<Vec<&Project>>> {
        let mut levels: Vec<Vec<&Project>> = Vec::new();
        let mut remaining: Vec<&Project> = self.projects.iter().collect();
        let mut completed: HashSet<String> = HashSet::new();

        while !remaining.is_empty() {
            let (ready, not_ready): (Vec<_>, Vec<_>) = remaining.into_iter().partition(|p| {
                p.depends_on
                    .as_ref()
                    .map(|deps| deps.iter().all(|d| completed.contains(d)))
                    .unwrap_or(true)
            });

            if ready.is_empty() {
                let names: Vec<_> = not_ready.iter().map(|p| p.name.as_str()).collect();
                anyhow::bail!(
                    "Cyclic or unresolvable dependencies among: {}",
                    names.join(", ")
                );
            }

            for p in &ready {
                completed.insert(p.name.clone());
            }
            levels.push(ready);
            remaining = not_ready;
        }

        Ok(levels)
    }

    /// Builds a map from project name → list of projects that depend on it.
    pub fn dependent_map(&self) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for project in &self.projects {
            map.entry(project.name.clone()).or_default();
            if let Some(deps) = &project.depends_on {
                for dep in deps {
                    map.entry(dep.clone())
                        .or_default()
                        .push(project.name.clone());
                }
            }
        }
        map
    }
}
