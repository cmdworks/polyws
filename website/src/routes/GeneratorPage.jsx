import React, { useMemo, useState } from "react";
import { Link } from "react-router-dom";
import {
  Folder,
  FolderOpen,
  GitBranch,
  Pencil,
  PlusCircle,
  Save,
  Trash2,
  XCircle,
} from "lucide-react";
import Footer from "../Footer";
import Navbar from "../Navbar";

const FILE_NAMES = [
  ".polyws",
  ".poly",
  ".polyws.json",
  ".poly.json",
  ".polyws.toml",
  ".poly.toml",
];

function emptyProject() {
  return {
    name: "",
    path: "",
    url: "",
    branch: "main",
    dependsOnCsv: "",
    syncUrl: "",
    syncInterval: "",
  };
}

function normalizePath(value) {
  return String(value)
    .trim()
    .replace(/\\/g, "/")
    .replace(/^\/+/, "")
    .replace(/\/+$/, "")
    .replace(/\/{2,}/g, "/");
}

function csvToArray(value) {
  return String(value)
    .split(",")
    .map((v) => v.trim())
    .filter(Boolean);
}

function slugify(value) {
  return String(value)
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9/_-]+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");
}

function escToml(value) {
  return String(value).replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}

function toToml(cfg) {
  const lines = [];
  lines.push(`name = "${escToml(cfg.name)}"`);

  if (typeof cfg.sync_interval_minutes === "number") {
    lines.push(`sync_interval_minutes = ${cfg.sync_interval_minutes}`);
  }

  if (!cfg.projects.length) {
    lines.push("projects = []");
  }

  cfg.projects.forEach((p) => {
    lines.push("");
    lines.push("[[projects]]");
    lines.push(`name = "${escToml(p.name)}"`);
    if (p.path) lines.push(`path = "${escToml(p.path)}"`);
    lines.push(`url = "${escToml(p.url)}"`);
    lines.push(`branch = "${escToml(p.branch || "main")}"`);

    if (Array.isArray(p.depends_on) && p.depends_on.length) {
      lines.push(
        `depends_on = [${p.depends_on.map((d) => `"${escToml(d)}"`).join(", ")}]`,
      );
    }

    if (p.sync_url) lines.push(`sync_url = "${escToml(p.sync_url)}"`);
    if (typeof p.sync_interval === "number") {
      lines.push(`sync_interval = ${p.sync_interval}`);
    }
  });

  if (cfg.vm) {
    lines.push("");
    lines.push("[vm]");
    lines.push(`host = "${escToml(cfg.vm.host)}"`);
    lines.push(`user = "${escToml(cfg.vm.user)}"`);
    lines.push(`path = "${escToml(cfg.vm.path)}"`);
    lines.push(`sync = "${escToml(cfg.vm.sync || "rsync")}"`);
    if (Array.isArray(cfg.vm.dependencies) && cfg.vm.dependencies.length) {
      lines.push(
        `dependencies = [${cfg.vm.dependencies.map((d) => `"${escToml(d)}"`).join(", ")}]`,
      );
    }
  }

  return `${lines.join("\n")}\n`;
}

function buildTreePreview(workspaceName, nodes) {
  const childMap = new Map();
  nodes.forEach((node) => {
    const key = node.parentId || "root";
    if (!childMap.has(key)) {
      childMap.set(key, []);
    }
    childMap.get(key).push(node);
  });

  for (const list of childMap.values()) {
    list.sort((a, b) => a.name.localeCompare(b.name));
  }

  const lines = [`[${workspaceName || "workspace"}]`];

  const walk = (parentId, prefix) => {
    const entries = childMap.get(parentId || "root") || [];

    entries.forEach((node, index) => {
      const isLast = index === entries.length - 1;
      const connector = isLast ? "└── " : "├── ";
      const name = node.kind === "folder" ? `${node.name}/` : `${node.name}/`;
      const repoInfo =
        node.kind === "repo"
          ? `  [repo: ${node.repoName || node.name}]`
          : "";

      lines.push(`${prefix}${connector}${name}${repoInfo}`);

      if (node.kind === "folder") {
        walk(node.id, `${prefix}${isLast ? "    " : "│   "}`);
      }
    });
  };

  walk(null, "");

  if (lines.length === 1) {
    lines.push("└── (add folders/projects)");
  }

  return `${lines.join("\n")}\n`;
}

function collectDescendantIds(nodes, nodeId) {
  const direct = nodes.filter((n) => n.parentId === nodeId);
  const nested = direct.flatMap((n) => collectDescendantIds(nodes, n.id));
  return [nodeId, ...nested];
}

function makeNode(id, kind, parentId = null) {
  const base = kind === "repo" ? "project" : "folder";
  return {
    id,
    kind,
    parentId,
    name: `${base}-${id.split("-").pop()}`,
    repoName: kind === "repo" ? `${base}-${id.split("-").pop()}` : "",
  };
}

export default function GeneratorPage() {
  const [workspaceName, setWorkspaceName] = useState("my-workspace");
  const [defaultInterval, setDefaultInterval] = useState("");
  const [format, setFormat] = useState("json");
  const [filename, setFilename] = useState(".polyws");

  const [projects, setProjects] = useState([emptyProject()]);

  const [vmEnabled, setVmEnabled] = useState(false);
  const [vmHost, setVmHost] = useState("");
  const [vmUser, setVmUser] = useState("");
  const [vmPath, setVmPath] = useState("");
  const [vmSync, setVmSync] = useState("mutagen");
  const [vmDeps, setVmDeps] = useState("");

  const [nodeCounter, setNodeCounter] = useState(5);
  const [plannerNodes, setPlannerNodes] = useState([
    { id: "node-1", kind: "folder", parentId: null, name: "apps", repoName: "" },
    {
      id: "node-2",
      kind: "repo",
      parentId: null,
      name: "project-1",
      repoName: "project-1",
    },
    { id: "node-3", kind: "folder", parentId: null, name: "services", repoName: "" },
    { id: "node-4", kind: "folder", parentId: "node-3", name: "api", repoName: "" },
  ]);

  const [editorMode, setEditorMode] = useState("workspace");
  const [selectedNodeId, setSelectedNodeId] = useState(null);
  const [nodeForm, setNodeForm] = useState({
    id: null,
    name: "",
    kind: "folder",
    parentId: "",
    repoName: "",
  });

  const { config, skippedRows } = useMemo(() => {
    const normalized = projects.map((p) => {
      const name = p.name.trim();
      const url = p.url.trim();
      if (!name || !url) return null;

      const out = {
        name,
        url,
        branch: p.branch.trim() || "main",
      };

      const path = normalizePath(p.path);
      if (path && path !== name) out.path = path;

      const deps = csvToArray(p.dependsOnCsv);
      if (deps.length) out.depends_on = deps;

      const syncUrl = p.syncUrl.trim();
      if (syncUrl) out.sync_url = syncUrl;

      const interval = Number(p.syncInterval);
      if (Number.isFinite(interval) && interval > 0) {
        out.sync_interval = interval;
      }

      return out;
    });

    const cleanProjects = normalized.filter(Boolean);

    const cfg = {
      name: workspaceName.trim() || "workspace",
      projects: cleanProjects,
    };

    const syncIntervalMinutes = Number(defaultInterval);
    if (Number.isFinite(syncIntervalMinutes) && syncIntervalMinutes > 0) {
      cfg.sync_interval_minutes = syncIntervalMinutes;
    }

    if (vmEnabled) {
      const host = vmHost.trim();
      const user = vmUser.trim();
      const path = vmPath.trim();
      if (host && user && path) {
        cfg.vm = {
          host,
          user,
          path,
          sync: vmSync.trim() || "rsync",
        };
        const deps = csvToArray(vmDeps);
        if (deps.length) cfg.vm.dependencies = deps;
      }
    }

    return {
      config: cfg,
      skippedRows: normalized.length - cleanProjects.length,
    };
  }, [
    projects,
    workspaceName,
    defaultInterval,
    vmEnabled,
    vmHost,
    vmUser,
    vmPath,
    vmSync,
    vmDeps,
  ]);

  const warnings = useMemo(() => {
    const msgs = [];

    if (skippedRows > 0) {
      msgs.push(`${skippedRows} incomplete project row(s) skipped.`);
    }

    const seen = new Map();
    config.projects.forEach((p) => {
      const effective = normalizePath(p.path || p.name);
      if (!seen.has(effective)) seen.set(effective, []);
      seen.get(effective).push(p.name);
    });

    for (const [path, names] of seen.entries()) {
      if (names.length > 1) {
        msgs.push(`duplicate path '${path}' used by: ${names.join(", ")}`);
      }
    }

    return msgs;
  }, [config.projects, skippedRows]);

  const output = useMemo(() => {
    if (format === "json") {
      return `${JSON.stringify(config, null, 2)}\n`;
    }
    return toToml(config);
  }, [config, format]);

  const treePreview = useMemo(
    () => buildTreePreview(workspaceName.trim() || "workspace", plannerNodes),
    [workspaceName, plannerNodes],
  );

  const rootFolderOptions = useMemo(() => {
    const folders = plannerNodes.filter((n) => n.kind === "folder");
    return folders.sort((a, b) => a.name.localeCompare(b.name));
  }, [plannerNodes]);

  const startWorkspaceEditor = () => {
    setEditorMode("workspace");
    setSelectedNodeId(null);
  };

  const startNodeEditor = (nodeId) => {
    const node = plannerNodes.find((n) => n.id === nodeId);
    if (!node) return;

    setEditorMode("node");
    setSelectedNodeId(node.id);
    setNodeForm({
      id: node.id,
      name: node.name,
      kind: node.kind,
      parentId: node.parentId || "",
      repoName: node.repoName || "",
    });
  };

  const createNode = (parentId, kind) => {
    const id = `node-${nodeCounter}`;
    setNodeCounter((n) => n + 1);
    const next = makeNode(id, kind, parentId);

    setPlannerNodes((prev) => [...prev, next]);
    setEditorMode("node");
    setSelectedNodeId(next.id);
    setNodeForm({
      id: next.id,
      name: next.name,
      kind: next.kind,
      parentId: next.parentId || "",
      repoName: next.repoName,
    });
  };

  const saveNode = () => {
    if (!nodeForm.id) return;

    const cleanName = nodeForm.name.trim();
    if (!cleanName) return;

    const cleanRepoName = nodeForm.repoName.trim();

    setPlannerNodes((prev) =>
      prev.map((node) => {
        if (node.id !== nodeForm.id) return node;
        return {
          ...node,
          name: cleanName,
          kind: nodeForm.kind,
          parentId: nodeForm.parentId || null,
          repoName:
            nodeForm.kind === "repo"
              ? cleanRepoName || cleanName
              : "",
        };
      }),
    );
  };

  const deleteNode = (nodeId) => {
    const ids = new Set(collectDescendantIds(plannerNodes, nodeId));
    setPlannerNodes((prev) => prev.filter((n) => !ids.has(n.id)));

    if (selectedNodeId && ids.has(selectedNodeId)) {
      setSelectedNodeId(null);
      setEditorMode("workspace");
    }
  };

  const updateProject = (idx, key, value) => {
    setProjects((prev) =>
      prev.map((p, i) => (i === idx ? { ...p, [key]: value } : p)),
    );
  };

  const removeProject = (idx) => {
    setProjects((prev) => {
      const next = prev.filter((_, i) => i !== idx);
      return next.length ? next : [emptyProject()];
    });
  };

  const copyOutput = async () => {
    try {
      await navigator.clipboard.writeText(output);
    } catch {
      // noop
    }
  };

  const downloadOutput = () => {
    const blob = new Blob([output], { type: "text/plain;charset=utf-8" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = filename || ".polyws";
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(a.href);
  };

  const childMap = useMemo(() => {
    const map = new Map();
    plannerNodes.forEach((node) => {
      const key = node.parentId || "root";
      if (!map.has(key)) map.set(key, []);
      map.get(key).push(node);
    });
    for (const list of map.values()) {
      list.sort((a, b) => a.name.localeCompare(b.name));
    }
    return map;
  }, [plannerNodes]);

  const renderTreeRows = (parentId, prefix = "") => {
    const nodes = childMap.get(parentId || "root") || [];

    return nodes.map((node, idx) => {
      const isLast = idx === nodes.length - 1;
      const connector = isLast ? "└──" : "├──";
      const branchPrefix = `${prefix}${connector}`;
      const nextPrefix = `${prefix}${isLast ? "    " : "│   "}`;

      return (
        <React.Fragment key={node.id}>
          <div className="flex items-center justify-between gap-2 rounded-lg border border-white/10 bg-black/25 px-2 py-2">
            <div className="min-w-0 flex items-center gap-2">
              <span className="text-muted text-xs shrink-0">{branchPrefix}</span>
              <button
                type="button"
                onClick={() => startNodeEditor(node.id)}
                className="text-primary hover:text-white shrink-0"
                title="Edit node"
              >
                {node.kind === "folder" ? <Folder size={16} /> : <GitBranch size={16} />}
              </button>
              <span className="text-xs text-foreground truncate">
                {node.kind === "folder" ? `${node.name}/` : `${node.name}/`}
                {node.kind === "repo" ? (
                  <span className="text-cyan-300"> {`[repo: ${node.repoName || node.name}]`}</span>
                ) : null}
              </span>
            </div>
            <div className="flex items-center gap-1 shrink-0">
              <button
                type="button"
                onClick={() => startNodeEditor(node.id)}
                className="p-1 rounded border border-white/20 text-muted hover:text-white"
                title="Edit"
              >
                <Pencil size={14} />
              </button>
              <button
                type="button"
                onClick={() => createNode(node.kind === "folder" ? node.id : node.parentId, "folder")}
                className="p-1 rounded border border-white/20 text-muted hover:text-white"
                title="Add nested folder"
              >
                <PlusCircle size={14} />
              </button>
              <button
                type="button"
                onClick={() => deleteNode(node.id)}
                className="p-1 rounded border border-tertiary/40 text-tertiary hover:bg-tertiary/10"
                title="Delete"
              >
                <Trash2 size={14} />
              </button>
            </div>
          </div>

          {node.kind === "folder" ? (
            <div className="pl-2 space-y-2">{renderTreeRows(node.id, nextPrefix)}</div>
          ) : null}
        </React.Fragment>
      );
    });
  };

  const base = import.meta.env.BASE_URL;

  return (
    <div className="min-h-screen bg-background text-foreground font-mono relative overflow-x-hidden">
      <Navbar />

      <main className="max-w-[1320px] mx-auto px-4 pt-28 pb-8">
        <header className="flex flex-wrap items-center justify-between gap-3 mb-4">
          <div>
            <h1 className="text-2xl md:text-3xl font-bold tracking-wide text-gradient-primary">
              polyws Config Generator
            </h1>
            <p className="text-xs text-muted mt-1">
              Build JSON/TOML config and visually plan nested folder/repo structure.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2 text-xs uppercase tracking-wider">
            <Link
              to="/docs"
              className="px-3 py-2 border border-primary/30 rounded-lg hover:bg-primary/10 transition"
            >
              Docs
            </Link>
            <a
              href={`${base}generet`}
              className="px-3 py-2 bg-gradient-primary text-black rounded-lg font-semibold"
            >
              /polyws/generet
            </a>
          </div>
        </header>

        <div className="grid grid-cols-1 xl:grid-cols-[1.2fr_1fr] gap-4">
          <section className="bg-surface/90 border border-white/15 rounded-2xl p-4 space-y-4">
            <div>
              <h2 className="text-primary text-xs uppercase tracking-[0.18em] mb-3">Workspace Config</h2>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <label className="text-xs text-muted uppercase tracking-wider">
                  Name
                  <input
                    className="mt-1 w-full bg-black/35 border border-white/20 rounded-lg px-3 py-2 text-sm"
                    value={workspaceName}
                    onChange={(e) => setWorkspaceName(e.target.value)}
                  />
                </label>
                <label className="text-xs text-muted uppercase tracking-wider">
                  sync_interval_minutes (optional)
                  <input
                    className="mt-1 w-full bg-black/35 border border-white/20 rounded-lg px-3 py-2 text-sm"
                    type="number"
                    min="1"
                    placeholder="30"
                    value={defaultInterval}
                    onChange={(e) => setDefaultInterval(e.target.value)}
                  />
                </label>
                <label className="text-xs text-muted uppercase tracking-wider">
                  Output Format
                  <select
                    className="mt-1 w-full bg-black/35 border border-white/20 rounded-lg px-3 py-2 text-sm"
                    value={format}
                    onChange={(e) => {
                      const nextFormat = e.target.value;
                      setFormat(nextFormat);
                      if (nextFormat === "json" && filename.endsWith(".toml")) {
                        setFilename(".polyws.json");
                      } else if (
                        nextFormat === "toml" &&
                        filename.endsWith(".json")
                      ) {
                        setFilename(".polyws.toml");
                      }
                    }}
                  >
                    <option value="json">JSON</option>
                    <option value="toml">TOML</option>
                  </select>
                </label>
                <label className="text-xs text-muted uppercase tracking-wider">
                  Suggested Filename
                  <select
                    className="mt-1 w-full bg-black/35 border border-white/20 rounded-lg px-3 py-2 text-sm"
                    value={filename}
                    onChange={(e) => setFilename(e.target.value)}
                  >
                    {FILE_NAMES.map((f) => (
                      <option key={f} value={f}>
                        {f}
                      </option>
                    ))}
                  </select>
                </label>
              </div>
            </div>

            <div className="h-px bg-white/10" />

            <div>
              <div className="flex items-center justify-between mb-2">
                <h2 className="text-primary text-xs uppercase tracking-[0.18em]">Projects</h2>
                <button
                  type="button"
                  className="px-3 py-2 text-xs uppercase tracking-wider border border-white/25 rounded-lg hover:bg-white/5"
                  onClick={() => setProjects((prev) => [...prev, emptyProject()])}
                >
                  Add Project
                </button>
              </div>
              <p className="text-xs text-muted mb-2">
                Use nested paths like <code className="text-cyan-300">apps/platform/core</code>.
              </p>

              <div className="space-y-3">
                {projects.map((p, idx) => (
                  <div key={idx} className="border border-white/15 rounded-xl p-3 bg-black/25">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-[11px] text-primary uppercase tracking-[0.14em]">
                        Project #{idx + 1}
                      </span>
                      <div className="flex gap-2">
                        <button
                          type="button"
                          className="px-2 py-1 text-[10px] uppercase border border-white/20 rounded hover:bg-white/5"
                          onClick={() =>
                            updateProject(idx, "path", normalizePath(slugify(p.name)))
                          }
                        >
                          Path From Name
                        </button>
                        <button
                          type="button"
                          className="px-2 py-1 text-[10px] uppercase border border-tertiary/40 text-tertiary rounded hover:bg-tertiary/10"
                          onClick={() => removeProject(idx)}
                          disabled={projects.length === 1}
                        >
                          Remove
                        </button>
                      </div>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                      {[
                        ["name", "Name", "core"],
                        ["path", "Path (nested)", "apps/platform/core"],
                        ["url", "URL", "git@github.com:org/core.git"],
                        ["branch", "Branch", "main"],
                        ["dependsOnCsv", "depends_on (csv)", "core,plugins"],
                        ["syncUrl", "sync_url (optional)", "git@gitlab.com:backup/core.git"],
                        ["syncInterval", "sync_interval (optional)", "10"],
                      ].map(([key, label, placeholder]) => (
                        <label
                          key={key}
                          className="text-[11px] text-muted uppercase tracking-wider"
                        >
                          {label}
                          <input
                            className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                            type={key === "syncInterval" ? "number" : "text"}
                            min={key === "syncInterval" ? "1" : undefined}
                            placeholder={placeholder}
                            value={p[key]}
                            onChange={(e) => updateProject(idx, key, e.target.value)}
                          />
                        </label>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="h-px bg-white/10" />

            <div>
              <div className="flex items-center justify-between mb-2">
                <h2 className="text-primary text-xs uppercase tracking-[0.18em]">Workspace Tree Planner</h2>
                <div className="flex gap-2">
                  <button
                    type="button"
                    className="px-2 py-1 text-[10px] uppercase border border-white/20 rounded hover:bg-white/5"
                    onClick={() => createNode(null, "folder")}
                  >
                    Add Folder
                  </button>
                  <button
                    type="button"
                    className="px-2 py-1 text-[10px] uppercase border border-white/20 rounded hover:bg-white/5"
                    onClick={() => createNode(null, "repo")}
                  >
                    Add Repo
                  </button>
                </div>
              </div>

              <div className="space-y-2">
                <div className="flex items-center justify-between gap-2 rounded-lg border border-primary/30 bg-black/30 px-2 py-2">
                  <div className="min-w-0 flex items-center gap-2">
                    <button
                      type="button"
                      onClick={startWorkspaceEditor}
                      className="text-primary hover:text-white"
                      title="Edit workspace"
                    >
                      <FolderOpen size={16} />
                    </button>
                    <span className="text-xs text-foreground truncate">
                      [{workspaceName || "workspace"}]
                    </span>
                  </div>
                  <button
                    type="button"
                    onClick={startWorkspaceEditor}
                    className="p-1 rounded border border-white/20 text-muted hover:text-white"
                    title="Edit workspace"
                  >
                    <Pencil size={14} />
                  </button>
                </div>

                <div className="space-y-2">{renderTreeRows(null)}</div>
              </div>
            </div>

            <div className="h-px bg-white/10" />

            <div>
              <h2 className="text-primary text-xs uppercase tracking-[0.18em] mb-2">VM (optional)</h2>
              <label className="flex items-center gap-2 text-sm text-muted mb-3">
                <input
                  type="checkbox"
                  checked={vmEnabled}
                  onChange={(e) => setVmEnabled(e.target.checked)}
                />
                Include vm section
              </label>

              {vmEnabled ? (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                  <label className="text-[11px] text-muted uppercase tracking-wider">
                    vm.host
                    <input
                      className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                      value={vmHost}
                      onChange={(e) => setVmHost(e.target.value)}
                      placeholder="dev-box"
                    />
                  </label>
                  <label className="text-[11px] text-muted uppercase tracking-wider">
                    vm.user
                    <input
                      className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                      value={vmUser}
                      onChange={(e) => setVmUser(e.target.value)}
                      placeholder="ubuntu"
                    />
                  </label>
                  <label className="text-[11px] text-muted uppercase tracking-wider">
                    vm.path
                    <input
                      className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                      value={vmPath}
                      onChange={(e) => setVmPath(e.target.value)}
                      placeholder="~/workspace/my-workspace"
                    />
                  </label>
                  <label className="text-[11px] text-muted uppercase tracking-wider">
                    vm.sync
                    <select
                      className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                      value={vmSync}
                      onChange={(e) => setVmSync(e.target.value)}
                    >
                      <option value="mutagen">mutagen</option>
                      <option value="rsync">rsync</option>
                    </select>
                  </label>
                  <label className="text-[11px] text-muted uppercase tracking-wider md:col-span-2">
                    vm.dependencies (csv)
                    <input
                      className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                      value={vmDeps}
                      onChange={(e) => setVmDeps(e.target.value)}
                      placeholder="git,rust,cargo"
                    />
                  </label>
                </div>
              ) : null}
            </div>
          </section>

          <section className="space-y-4">
            <div className="bg-surface/90 border border-white/15 rounded-2xl p-4">
              <h2 className="text-primary text-xs uppercase tracking-[0.18em] mb-2">Generated Config</h2>
              <pre className="bg-black/70 border border-white/10 rounded-xl p-3 min-h-[280px] overflow-auto text-xs leading-6">
                {output}
              </pre>
              <div className="flex flex-wrap gap-2 mt-3">
                <button
                  type="button"
                  className="px-3 py-2 text-xs uppercase tracking-wider border border-white/25 rounded-lg hover:bg-white/5"
                  onClick={copyOutput}
                >
                  Copy
                </button>
                <button
                  type="button"
                  className="px-3 py-2 text-xs uppercase tracking-wider bg-gradient-primary text-black rounded-lg font-semibold"
                  onClick={downloadOutput}
                >
                  Download
                </button>
                <span className="px-2 py-2 text-[10px] uppercase tracking-wider border border-white/20 rounded-lg text-muted">
                  {filename}
                </span>
              </div>

              <div className="flex flex-wrap gap-2 mt-3">
                <span className="px-2 py-1 text-[10px] uppercase tracking-wider border border-emerald-400/40 text-emerald-300 rounded-full">
                  projects: {config.projects.length}
                </span>
                <span className="px-2 py-1 text-[10px] uppercase tracking-wider border border-white/20 text-muted rounded-full">
                  planner nodes: {plannerNodes.length}
                </span>
                {warnings.map((msg) => (
                  <span
                    key={msg}
                    className="px-2 py-1 text-[10px] uppercase tracking-wider border border-amber-400/40 text-amber-300 rounded-full"
                  >
                    {msg}
                  </span>
                ))}
              </div>

              <p className="text-[11px] text-muted mt-3 leading-5">
                Valid config names: {FILE_NAMES.join(", ")}
              </p>
            </div>

            <div className="bg-surface/90 border border-white/15 rounded-2xl p-4">
              <h2 className="text-primary text-xs uppercase tracking-[0.18em] mb-2">Node Editor</h2>
              {editorMode === "workspace" ? (
                <div className="space-y-3">
                  <label className="text-[11px] text-muted uppercase tracking-wider block">
                    Workspace name
                    <input
                      className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                      value={workspaceName}
                      onChange={(e) => setWorkspaceName(e.target.value)}
                    />
                  </label>
                  <div className="flex flex-wrap gap-2">
                    <button
                      type="button"
                      className="px-3 py-2 text-xs uppercase tracking-wider border border-white/20 rounded-lg hover:bg-white/5"
                      onClick={() => createNode(null, "folder")}
                    >
                      Add Root Folder
                    </button>
                    <button
                      type="button"
                      className="px-3 py-2 text-xs uppercase tracking-wider border border-white/20 rounded-lg hover:bg-white/5"
                      onClick={() => createNode(null, "repo")}
                    >
                      Add Root Repo
                    </button>
                  </div>
                  <p className="text-[11px] text-muted">
                    Click any folder/repo icon in the tree to edit, delete, or add nested folders.
                  </p>
                </div>
              ) : (
                <div className="space-y-3">
                  <label className="text-[11px] text-muted uppercase tracking-wider block">
                    Node Type
                    <select
                      className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                      value={nodeForm.kind}
                      onChange={(e) =>
                        setNodeForm((prev) => ({
                          ...prev,
                          kind: e.target.value,
                          repoName:
                            e.target.value === "repo"
                              ? prev.repoName || prev.name
                              : "",
                        }))
                      }
                    >
                      <option value="folder">folder</option>
                      <option value="repo">repo</option>
                    </select>
                  </label>

                  <label className="text-[11px] text-muted uppercase tracking-wider block">
                    Name
                    <input
                      className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                      value={nodeForm.name}
                      onChange={(e) =>
                        setNodeForm((prev) => ({ ...prev, name: e.target.value }))
                      }
                    />
                  </label>

                  {nodeForm.kind === "repo" ? (
                    <label className="text-[11px] text-muted uppercase tracking-wider block">
                      Repo Label
                      <input
                        className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                        value={nodeForm.repoName}
                        onChange={(e) =>
                          setNodeForm((prev) => ({ ...prev, repoName: e.target.value }))
                        }
                        placeholder="project-1"
                      />
                    </label>
                  ) : null}

                  <label className="text-[11px] text-muted uppercase tracking-wider block">
                    Parent folder
                    <select
                      className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                      value={nodeForm.parentId}
                      onChange={(e) =>
                        setNodeForm((prev) => ({ ...prev, parentId: e.target.value }))
                      }
                    >
                      <option value="">[workspace root]</option>
                      {rootFolderOptions
                        .filter((f) => f.id !== nodeForm.id)
                        .map((folder) => (
                          <option key={folder.id} value={folder.id}>
                            {folder.name}
                          </option>
                        ))}
                    </select>
                  </label>

                  <div className="flex flex-wrap gap-2">
                    <button
                      type="button"
                      onClick={saveNode}
                      className="inline-flex items-center gap-1 px-3 py-2 text-xs uppercase tracking-wider bg-gradient-primary text-black rounded-lg font-semibold"
                    >
                      <Save size={14} />
                      Save
                    </button>
                    <button
                      type="button"
                      onClick={() => {
                        if (selectedNodeId) {
                          createNode(selectedNodeId, "folder");
                        }
                      }}
                      className="inline-flex items-center gap-1 px-3 py-2 text-xs uppercase tracking-wider border border-white/20 rounded-lg hover:bg-white/5"
                    >
                      <PlusCircle size={14} />
                      Add Nested Folder
                    </button>
                    <button
                      type="button"
                      onClick={() => {
                        if (selectedNodeId) {
                          deleteNode(selectedNodeId);
                        }
                      }}
                      className="inline-flex items-center gap-1 px-3 py-2 text-xs uppercase tracking-wider border border-tertiary/40 text-tertiary rounded-lg hover:bg-tertiary/10"
                    >
                      <Trash2 size={14} />
                      Delete
                    </button>
                    <button
                      type="button"
                      onClick={() => startWorkspaceEditor()}
                      className="inline-flex items-center gap-1 px-3 py-2 text-xs uppercase tracking-wider border border-white/20 rounded-lg hover:bg-white/5"
                    >
                      <XCircle size={14} />
                      Close
                    </button>
                  </div>
                </div>
              )}
            </div>

            <div className="bg-surface/90 border border-white/15 rounded-2xl p-4">
              <h2 className="text-primary text-xs uppercase tracking-[0.18em] mb-2">Workspace Tree Preview</h2>
              <pre className="bg-black/70 border border-white/10 rounded-xl p-3 min-h-[230px] overflow-auto text-xs leading-6">
                {treePreview}
              </pre>
              <p className="text-[11px] text-muted mt-2 leading-5">
                Example style: <code className="text-cyan-300">├── apps/</code>,{" "}
                <code className="text-cyan-300">└── services/</code>, and repo tags like{" "}
                <code className="text-cyan-300">[repo: project-1]</code>.
              </p>
            </div>
          </section>
        </div>
      </main>

      <Footer />
    </div>
  );
}
