import React, { useMemo, useState } from "react";
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

function collectDescendantIds(nodes, nodeId) {
  const direct = nodes.filter((n) => n.parentId === nodeId);
  const nested = direct.flatMap((n) => collectDescendantIds(nodes, n.id));
  return [nodeId, ...nested];
}

function makeNode(id, kind, parentId = null) {
  const serial = id.split("-").pop();
  const base = kind === "repo" ? "project" : "folder";
  return {
    id,
    kind,
    parentId,
    name: `${base}-${serial}`,
    repoName: kind === "repo" ? `${base}-${serial}` : "",
    url: "",
    syncUrl: "",
    branch: "main",
  };
}

function buildTreePreview(workspaceName, nodes) {
  const childMap = new Map();
  nodes.forEach((node) => {
    const key = node.parentId || "root";
    if (!childMap.has(key)) childMap.set(key, []);
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
      const repoInfo =
        node.kind === "repo"
          ? `  [repo: ${node.repoName || node.name}]`
          : "";

      lines.push(`${prefix}${connector}${node.name}/${repoInfo}`);

      if (node.kind === "folder") {
        walk(node.id, `${prefix}${isLast ? "    " : "│   "}`);
      }
    });
  };

  walk(null, "");

  if (lines.length === 1) {
    lines.push("└── (add folders and repos)");
  }

  return `${lines.join("\n")}\n`;
}

function pathForNode(node, nodeMap) {
  const names = [];
  let cursor = node;

  while (cursor) {
    if (cursor.name && cursor.name.trim()) {
      names.push(cursor.name.trim());
    }
    cursor = cursor.parentId ? nodeMap.get(cursor.parentId) : null;
  }

  return normalizePath(names.reverse().join("/"));
}

function deriveProjects(nodes) {
  const nodeMap = new Map(nodes.map((node) => [node.id, node]));
  const warnings = [];
  const projects = [];

  nodes
    .filter((node) => node.kind === "repo")
    .forEach((repoNode) => {
      const dirName = repoNode.name.trim();
      const projectName = (repoNode.repoName || repoNode.name).trim();
      const url = repoNode.url.trim();

      if (!dirName || !projectName || !url) {
        warnings.push(
          `repo '${repoNode.name || repoNode.id}' skipped (name/project/url required)`,
        );
        return;
      }

      const fullPath = pathForNode(repoNode, nodeMap);
      const project = {
        name: projectName,
        url,
        branch: repoNode.branch?.trim() || "main",
      };

      if (fullPath && fullPath !== projectName) {
        project.path = fullPath;
      }

      const syncUrl = repoNode.syncUrl.trim();
      if (syncUrl) {
        project.sync_url = syncUrl;
      }

      projects.push(project);
    });

  const pathSeen = new Map();
  projects.forEach((project) => {
    const effectivePath = normalizePath(project.path || project.name);
    if (!pathSeen.has(effectivePath)) pathSeen.set(effectivePath, []);
    pathSeen.get(effectivePath).push(project.name);
  });

  for (const [repoPath, names] of pathSeen.entries()) {
    if (names.length > 1) {
      warnings.push(`duplicate path '${repoPath}' used by: ${names.join(", ")}`);
    }
  }

  return { projects, warnings };
}

export default function GeneratorPage() {
  const [workspaceName, setWorkspaceName] = useState("my-workspace");
  const [defaultInterval, setDefaultInterval] = useState("");
  const [format, setFormat] = useState("json");
  const [filename, setFilename] = useState(".polyws");

  const [vmEnabled, setVmEnabled] = useState(false);
  const [vmHost, setVmHost] = useState("");
  const [vmUser, setVmUser] = useState("");
  const [vmPath, setVmPath] = useState("");
  const [vmSync, setVmSync] = useState("mutagen");
  const [vmDeps, setVmDeps] = useState("");

  const [plannerNodes, setPlannerNodes] = useState([
    {
      id: "node-1",
      kind: "folder",
      parentId: null,
      name: "apps",
      repoName: "",
      url: "",
      syncUrl: "",
      branch: "main",
    },
    {
      id: "node-2",
      kind: "repo",
      parentId: null,
      name: "core",
      repoName: "core",
      url: "git@github.com:org/core.git",
      syncUrl: "",
      branch: "main",
    },
    {
      id: "node-3",
      kind: "folder",
      parentId: null,
      name: "services",
      repoName: "",
      url: "",
      syncUrl: "",
      branch: "main",
    },
    {
      id: "node-4",
      kind: "folder",
      parentId: "node-3",
      name: "api",
      repoName: "",
      url: "",
      syncUrl: "",
      branch: "main",
    },
  ]);
  const [nodeCounter, setNodeCounter] = useState(5);

  const [isEditorOpen, setIsEditorOpen] = useState(false);
  const [editorMode, setEditorMode] = useState("workspace");
  const [selectedNodeId, setSelectedNodeId] = useState(null);
  const [nodeForm, setNodeForm] = useState({
    id: null,
    kind: "folder",
    parentId: "",
    name: "",
    repoName: "",
    url: "",
    syncUrl: "",
    branch: "main",
  });

  const { plannerProjects, plannerWarnings } = useMemo(() => {
    const derived = deriveProjects(plannerNodes);
    return {
      plannerProjects: derived.projects,
      plannerWarnings: derived.warnings,
    };
  }, [plannerNodes]);

  const config = useMemo(() => {
    const cfg = {
      name: workspaceName.trim() || "workspace",
      projects: plannerProjects,
    };

    const interval = Number(defaultInterval);
    if (Number.isFinite(interval) && interval > 0) {
      cfg.sync_interval_minutes = interval;
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

    return cfg;
  }, [
    workspaceName,
    plannerProjects,
    defaultInterval,
    vmEnabled,
    vmHost,
    vmUser,
    vmPath,
    vmSync,
    vmDeps,
  ]);

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
    return plannerNodes
      .filter((node) => node.kind === "folder")
      .sort((a, b) => a.name.localeCompare(b.name));
  }, [plannerNodes]);

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

  const openWorkspaceEditor = () => {
    setEditorMode("workspace");
    setSelectedNodeId(null);
    setIsEditorOpen(true);
  };

  const openNodeEditor = (nodeId) => {
    const node = plannerNodes.find((item) => item.id === nodeId);
    if (!node) return;

    setEditorMode("node");
    setSelectedNodeId(node.id);
    setNodeForm({
      id: node.id,
      kind: node.kind,
      parentId: node.parentId || "",
      name: node.name,
      repoName: node.repoName || "",
      url: node.url || "",
      syncUrl: node.syncUrl || "",
      branch: node.branch || "main",
    });
    setIsEditorOpen(true);
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
      kind: next.kind,
      parentId: next.parentId || "",
      name: next.name,
      repoName: next.repoName,
      url: next.url,
      syncUrl: next.syncUrl,
      branch: next.branch,
    });
    setIsEditorOpen(true);
  };

  const createChildNode = (kind) => {
    if (!selectedNodeId) return;
    const current = plannerNodes.find((n) => n.id === selectedNodeId);
    if (!current) return;

    const parentId = current.kind === "folder" ? current.id : current.parentId;
    createNode(parentId || null, kind);
  };

  const closeEditor = () => {
    setIsEditorOpen(false);
  };

  const saveNode = () => {
    if (!nodeForm.id) return;
    const cleanName = nodeForm.name.trim();
    if (!cleanName) return;

    const cleanRepoName = nodeForm.repoName.trim();

    setPlannerNodes((prev) =>
      prev.map((node) => {
        if (node.id !== nodeForm.id) return node;

        const kind = nodeForm.kind;
        return {
          ...node,
          kind,
          parentId: nodeForm.parentId || null,
          name: cleanName,
          repoName: kind === "repo" ? cleanRepoName || cleanName : "",
          url: kind === "repo" ? nodeForm.url.trim() : "",
          syncUrl: kind === "repo" ? nodeForm.syncUrl.trim() : "",
          branch: kind === "repo" ? nodeForm.branch.trim() || "main" : "main",
        };
      }),
    );

    setIsEditorOpen(false);
  };

  const deleteNode = (nodeId) => {
    const idsToDelete = new Set(collectDescendantIds(plannerNodes, nodeId));
    setPlannerNodes((prev) => prev.filter((node) => !idsToDelete.has(node.id)));

    if (selectedNodeId && idsToDelete.has(selectedNodeId)) {
      setSelectedNodeId(null);
      setEditorMode("workspace");
      setIsEditorOpen(false);
    }
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

  const renderTreeRows = (parentId, depth = 0) => {
    const nodes = childMap.get(parentId || "root") || [];

    return nodes.map((node) => {
      const indent = depth * 14;
      const hasChildren = (childMap.get(node.id) || []).length > 0;

      return (
        <div key={node.id} className="space-y-0.5">
          <div
            style={{ marginLeft: `${indent}px` }}
            className="flex items-center justify-between gap-2 py-0.5"
          >
            <div className="min-w-0 flex items-center gap-2">
              <button
                type="button"
                onClick={() => openNodeEditor(node.id)}
                className="text-primary hover:text-white shrink-0"
                title="Edit node"
              >
                {node.kind === "folder" ? <Folder size={16} /> : <GitBranch size={16} />}
              </button>
              <span className="text-sm text-foreground truncate">
                {node.name}/
                {node.kind === "repo" ? (
                  <span className="text-cyan-300"> {`[repo: ${node.repoName || node.name}]`}</span>
                ) : null}
              </span>
            </div>

            <div className="flex items-center gap-1 shrink-0">
              <button
                type="button"
                onClick={() => openNodeEditor(node.id)}
                className="p-1 rounded border border-white/20 text-muted hover:text-white"
                title="Edit"
              >
                <Pencil size={14} />
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

          {node.kind === "folder" && hasChildren ? (
            <div
              style={{ marginLeft: `${indent + 8}px` }}
              className="border-l border-white/15 pl-2"
            >
              {renderTreeRows(node.id, depth + 1)}
            </div>
          ) : null}
        </div>
      );
    });
  };

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
              Planner-first config generation with live tree and live JSON/TOML output.
            </p>
          </div>
        </header>

        <div className="grid grid-cols-1 xl:grid-cols-[1.2fr_1fr] gap-4">
          <section className="bg-surface/90 border border-white/15 rounded-2xl p-4 space-y-4">
            <div>
              <h2 className="text-primary text-xs uppercase tracking-[0.18em] mb-2">Tree Preview</h2>
              <pre className="bg-black/70 border border-white/10 rounded-xl p-3 min-h-[180px] overflow-auto text-xs leading-6">
                {treePreview}
              </pre>
            </div>

            <div className="h-px bg-white/10" />

            <div>
              <div className="flex items-center justify-between mb-2">
                <h2 className="text-primary text-xs uppercase tracking-[0.18em]">Workspace Tree Planner</h2>
                <button
                  type="button"
                  className="px-2 py-1 text-[10px] uppercase border border-white/20 rounded hover:bg-white/5"
                  onClick={openWorkspaceEditor}
                >
                  Open Editor
                </button>
              </div>

              <div className="rounded-xl border border-white/15 bg-black/30 px-3 py-2">
                <div className="flex items-center justify-between gap-2">
                  <div className="min-w-0 flex items-center gap-2">
                    <button
                      type="button"
                      onClick={openWorkspaceEditor}
                      className="text-primary hover:text-white"
                      title="Edit workspace"
                    >
                      <FolderOpen size={16} />
                    </button>
                    <span className="text-sm text-foreground truncate">[{workspaceName || "workspace"}]</span>
                  </div>

                  <div className="flex items-center gap-1">
                    <button
                      type="button"
                      onClick={() => createNode(null, "folder")}
                      className="p-1 rounded border border-white/20 text-muted hover:text-white"
                      title="Add root folder"
                    >
                      <PlusCircle size={14} />
                    </button>
                    <button
                      type="button"
                      onClick={openWorkspaceEditor}
                      className="p-1 rounded border border-white/20 text-muted hover:text-white"
                      title="Edit workspace"
                    >
                      <Pencil size={14} />
                    </button>
                  </div>
                </div>

                <div className="mt-0.5">{renderTreeRows(null)}</div>
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

              <div className="grid grid-cols-1 md:grid-cols-3 gap-2 mb-3">
                <label className="text-[11px] text-muted uppercase tracking-wider md:col-span-1">
                  Output format
                  <select
                    className="mt-1 w-full bg-black/35 border border-white/20 rounded-lg px-3 py-2 text-sm"
                    value={format}
                    onChange={(e) => {
                      const nextFormat = e.target.value;
                      setFormat(nextFormat);
                      if (nextFormat === "json" && filename.endsWith(".toml")) {
                        setFilename(".polyws.json");
                      } else if (nextFormat === "toml" && filename.endsWith(".json")) {
                        setFilename(".polyws.toml");
                      }
                    }}
                  >
                    <option value="json">JSON</option>
                    <option value="toml">TOML</option>
                  </select>
                </label>
                <label className="text-[11px] text-muted uppercase tracking-wider md:col-span-2">
                  Suggested filename
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

              <pre className="bg-black/70 border border-white/10 rounded-xl p-3 min-h-[300px] overflow-auto text-xs leading-6">
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
                  repos: {plannerProjects.length}
                </span>
                <span className="px-2 py-1 text-[10px] uppercase tracking-wider border border-white/20 text-muted rounded-full">
                  planner nodes: {plannerNodes.length}
                </span>
                {plannerWarnings.map((msg) => (
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
          </section>
        </div>

        {isEditorOpen ? (
          <div className="fixed inset-0 z-[60] bg-black/75 backdrop-blur-sm flex items-center justify-center p-4">
            <div className="w-full max-w-2xl bg-surface border border-white/20 rounded-2xl p-4">
              <div className="flex items-center justify-between mb-3">
                <h2 className="text-primary text-sm uppercase tracking-[0.16em]">
                  {editorMode === "workspace" ? "Workspace Editor" : "Node Editor"}
                </h2>
                <button
                  type="button"
                  className="p-1 rounded border border-white/20 text-muted hover:text-white"
                  onClick={closeEditor}
                  title="Close editor"
                >
                  <XCircle size={16} />
                </button>
              </div>

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

                  <label className="text-[11px] text-muted uppercase tracking-wider block">
                    sync_interval_minutes (optional)
                    <input
                      className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                      type="number"
                      min="1"
                      value={defaultInterval}
                      onChange={(e) => setDefaultInterval(e.target.value)}
                    />
                  </label>

                  <div className="flex flex-wrap gap-2">
                    <button
                      type="button"
                      className="px-3 py-2 text-xs uppercase tracking-wider border border-white/20 rounded-lg hover:bg-white/5"
                      onClick={() => createNode(null, "folder")}
                    >
                      + Root Folder
                    </button>
                    <button
                      type="button"
                      className="px-3 py-2 text-xs uppercase tracking-wider border border-white/20 rounded-lg hover:bg-white/5"
                      onClick={() => createNode(null, "repo")}
                    >
                      + Root Repo
                    </button>
                  </div>
                </div>
              ) : (
                <div className="space-y-3">
                  <label className="text-[11px] text-muted uppercase tracking-wider block">
                    Node type
                    <select
                      className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                      value={nodeForm.kind}
                      onChange={(e) =>
                        setNodeForm((prev) => ({
                          ...prev,
                          kind: e.target.value,
                          repoName: e.target.value === "repo" ? prev.repoName || prev.name : "",
                          url: e.target.value === "repo" ? prev.url : "",
                          syncUrl: e.target.value === "repo" ? prev.syncUrl : "",
                          branch: e.target.value === "repo" ? prev.branch || "main" : "main",
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
                      onChange={(e) => setNodeForm((prev) => ({ ...prev, name: e.target.value }))}
                    />
                  </label>

                  {nodeForm.kind === "repo" ? (
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                      <label className="text-[11px] text-muted uppercase tracking-wider block">
                        Project name
                        <input
                          className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                          value={nodeForm.repoName}
                          onChange={(e) =>
                            setNodeForm((prev) => ({ ...prev, repoName: e.target.value }))
                          }
                          placeholder="core"
                        />
                      </label>
                      <label className="text-[11px] text-muted uppercase tracking-wider block">
                        Branch
                        <input
                          className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                          value={nodeForm.branch}
                          onChange={(e) => setNodeForm((prev) => ({ ...prev, branch: e.target.value }))}
                          placeholder="main"
                        />
                      </label>
                      <label className="text-[11px] text-muted uppercase tracking-wider block md:col-span-2">
                        Remote repo URL
                        <input
                          className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                          value={nodeForm.url}
                          placeholder="git@github.com:org/repo.git"
                          onChange={(e) => setNodeForm((prev) => ({ ...prev, url: e.target.value }))}
                        />
                      </label>
                      <label className="text-[11px] text-muted uppercase tracking-wider block md:col-span-2">
                        Sync repo URL (optional)
                        <input
                          className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                          value={nodeForm.syncUrl}
                          placeholder="git@gitlab.com:backup/repo.git"
                          onChange={(e) =>
                            setNodeForm((prev) => ({ ...prev, syncUrl: e.target.value }))
                          }
                        />
                      </label>
                    </div>
                  ) : null}

                  <label className="text-[11px] text-muted uppercase tracking-wider block">
                    Parent folder
                    <select
                      className="mt-1 w-full bg-black/30 border border-white/15 rounded-lg px-3 py-2 text-sm"
                      value={nodeForm.parentId}
                      onChange={(e) => setNodeForm((prev) => ({ ...prev, parentId: e.target.value }))}
                    >
                      <option value="">[workspace root]</option>
                      {rootFolderOptions
                        .filter((folder) => folder.id !== nodeForm.id)
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
                      onClick={() => createChildNode("folder")}
                      className="inline-flex items-center gap-1 px-3 py-2 text-xs uppercase tracking-wider border border-white/20 rounded-lg hover:bg-white/5"
                    >
                      <PlusCircle size={14} />
                      Add Child Folder
                    </button>
                    <button
                      type="button"
                      onClick={() => createChildNode("repo")}
                      className="inline-flex items-center gap-1 px-3 py-2 text-xs uppercase tracking-wider border border-white/20 rounded-lg hover:bg-white/5"
                    >
                      <PlusCircle size={14} />
                      Add Child Repo
                    </button>
                    <button
                      type="button"
                      onClick={() => selectedNodeId && deleteNode(selectedNodeId)}
                      className="inline-flex items-center gap-1 px-3 py-2 text-xs uppercase tracking-wider border border-tertiary/40 text-tertiary rounded-lg hover:bg-tertiary/10"
                    >
                      <Trash2 size={14} />
                      Delete
                    </button>
                  </div>
                </div>
              )}
            </div>
          </div>
        ) : null}
      </main>

      <Footer />
    </div>
  );
}
