use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
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

impl Project {
    /// Local directory for this project, relative to workspace root.
    /// Falls back to `name` for backward compatibility.
    pub fn local_dir(&self) -> &str {
        self.path.as_deref().unwrap_or(&self.name)
    }
}

/// Normalize a project path for comparisons.
/// - trims whitespace
/// - converts backslashes to slashes
/// - removes leading "./"
/// - trims trailing slashes
/// - treats empty as "."
pub fn normalize_local_dir(path: &str) -> String {
    let mut normalized = path.trim().replace('\\', "/");
    while normalized.starts_with("./") {
        normalized = normalized.trim_start_matches("./").to_string();
    }
    normalized = normalized.trim_end_matches('/').to_string();
    if normalized.is_empty() {
        ".".to_string()
    } else {
        normalized
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConfigFormat {
    Json,
    Toml,
}

const CONFIG_CANDIDATES: [&str; 6] = [
    ".polyws",
    ".poly",
    ".polyws.json",
    ".poly.json",
    ".polyws.toml",
    ".poly.toml",
];

pub fn known_config_paths() -> &'static [&'static str] {
    &CONFIG_CANDIDATES
}

pub fn find_existing_config_path() -> Option<&'static str> {
    CONFIG_CANDIDATES
        .iter()
        .copied()
        .find(|path| Path::new(path).is_file())
}

fn format_from_path(path: &str) -> Option<ConfigFormat> {
    if path.ends_with(".json") {
        Some(ConfigFormat::Json)
    } else if path.ends_with(".toml") {
        Some(ConfigFormat::Toml)
    } else {
        None
    }
}

fn parse_with_format(content: &str, format: ConfigFormat) -> Result<WorkspaceConfig> {
    match format {
        ConfigFormat::Json => serde_json::from_str(content).context("invalid JSON"),
        ConfigFormat::Toml => toml::from_str(content).context("invalid TOML"),
    }
}

fn parse_workspace_config(path: &str, content: &str) -> Result<(WorkspaceConfig, ConfigFormat)> {
    if let Some(format) = format_from_path(path) {
        let cfg = parse_with_format(content, format)?;
        return Ok((cfg, format));
    }

    if let Ok(cfg) = parse_with_format(content, ConfigFormat::Json) {
        return Ok((cfg, ConfigFormat::Json));
    }
    if let Ok(cfg) = parse_with_format(content, ConfigFormat::Toml) {
        return Ok((cfg, ConfigFormat::Toml));
    }

    anyhow::bail!("invalid config: expected JSON or TOML")
}

fn pick_default_save_target() -> Result<(&'static str, ConfigFormat)> {
    const DEFAULT_TARGETS: [(&str, ConfigFormat); 6] = [
        (".polyws", ConfigFormat::Json),
        (".poly", ConfigFormat::Json),
        (".polyws.json", ConfigFormat::Json),
        (".poly.json", ConfigFormat::Json),
        (".polyws.toml", ConfigFormat::Toml),
        (".poly.toml", ConfigFormat::Toml),
    ];

    for (path, format) in DEFAULT_TARGETS {
        let p = Path::new(path);
        if !p.exists() || p.is_file() {
            return Ok((path, format));
        }
    }

    anyhow::bail!("could not determine a writable workspace config path")
}

fn pick_save_target() -> Result<(&'static str, ConfigFormat)> {
    for path in CONFIG_CANDIDATES {
        let p = Path::new(path);
        if !p.is_file() {
            continue;
        }

        if let Some(format) = format_from_path(path) {
            return Ok((path, format));
        }

        // Extension-less file: keep whichever format it already uses.
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed reading existing config '{}'", path))?;
        let format = match parse_workspace_config(path, &content) {
            Ok((_, format)) => format,
            Err(_) => ConfigFormat::Json,
        };
        return Ok((path, format));
    }

    pick_default_save_target()
}

impl WorkspaceConfig {
    pub fn load() -> Result<Self> {
        let mut parse_errors = Vec::new();

        for path in CONFIG_CANDIDATES {
            let p = Path::new(path);
            if !p.exists() {
                continue;
            }
            if !p.is_file() {
                continue;
            }

            let content = fs::read_to_string(path)
                .with_context(|| format!("failed reading workspace config '{}'", path))?;
            match parse_workspace_config(path, &content) {
                Ok((cfg, _)) => return Ok(cfg),
                Err(e) => parse_errors.push(format!("{}: {}", path, e)),
            }
        }

        if !parse_errors.is_empty() {
            anyhow::bail!(
                "Found workspace config file(s), but none were valid:\n{}",
                parse_errors.join("\n")
            );
        }

        anyhow::bail!(
            "No workspace config found. Tried: {}",
            CONFIG_CANDIDATES.join(", ")
        )
    }

    pub fn save(&self) -> Result<()> {
        let (path, format) = pick_save_target()?;
        let content = match format {
            ConfigFormat::Json => serde_json::to_string_pretty(self)?,
            ConfigFormat::Toml => toml::to_string_pretty(self)?,
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

#[cfg(test)]
mod tests {
    use super::{parse_workspace_config, ConfigFormat};

    #[test]
    fn parses_json_from_extensionless_paths() {
        let content = r#"{"name":"demo","projects":[]}"#;
        let (cfg, format) = parse_workspace_config(".polyws", content).expect("json should parse");
        assert_eq!(cfg.name, "demo");
        assert_eq!(format, ConfigFormat::Json);
    }

    #[test]
    fn parses_toml_from_extensionless_paths() {
        let content = r#"name = "demo"
projects = []
"#;
        let (cfg, format) = parse_workspace_config(".poly", content).expect("toml should parse");
        assert_eq!(cfg.name, "demo");
        assert_eq!(format, ConfigFormat::Toml);
    }

    #[test]
    fn respects_explicit_toml_extension() {
        let content = r#"name = "demo"
projects = []
"#;
        let (cfg, format) =
            parse_workspace_config(".polyws.toml", content).expect("toml should parse");
        assert_eq!(cfg.name, "demo");
        assert_eq!(format, ConfigFormat::Toml);
    }

    #[test]
    fn respects_explicit_json_extension() {
        let content = r#"{"name":"demo","projects":[]}"#;
        let (cfg, format) =
            parse_workspace_config(".poly.json", content).expect("json should parse");
        assert_eq!(cfg.name, "demo");
        assert_eq!(format, ConfigFormat::Json);
    }
}
