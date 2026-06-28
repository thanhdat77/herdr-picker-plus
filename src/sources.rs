use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

use crate::{
    config::Config,
    herdr::herdr_json,
    model::{Entry, EntryAction, Source, WorkspaceKind, WorkspaceRef},
    paths::{basename, canonical_str, expand_path, home},
};

pub(crate) fn collect_workspaces() -> (Vec<Entry>, HashMap<String, Vec<WorkspaceRef>>) {
    let ws_json = herdr_json(["workspace", "list"]).unwrap_or(Value::Null);
    let pane_json = herdr_json(["pane", "list"]).unwrap_or(Value::Null);
    let mut cwd_by_ws: HashMap<String, String> = HashMap::new();
    if let Some(panes) = pane_json
        .pointer("/result/panes")
        .and_then(|v| v.as_array())
    {
        for p in panes {
            let Some(ws) = p.get("workspace_id").and_then(|v| v.as_str()) else {
                continue;
            };
            let cwd = p
                .get("foreground_cwd")
                .or_else(|| p.get("cwd"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !cwd.is_empty() {
                cwd_by_ws.entry(ws.into()).or_insert(cwd.into());
            }
        }
    }
    let mut entries = Vec::new();
    let mut map = HashMap::new();
    if let Some(workspaces) = ws_json
        .pointer("/result/workspaces")
        .and_then(|v| v.as_array())
    {
        for w in workspaces {
            let id = w.get("workspace_id").and_then(|v| v.as_str()).unwrap_or("");
            let label = w.get("label").and_then(|v| v.as_str()).unwrap_or(id);
            let cwd = cwd_by_ws
                .get(id)
                .cloned()
                .unwrap_or_else(|| home().display().to_string());
            let path = PathBuf::from(&cwd);
            let tab_count = w.get("tab_count").and_then(|v| v.as_i64()).unwrap_or(0);
            let pane_count = w.get("pane_count").and_then(|v| v.as_i64()).unwrap_or(0);
            if let Some(key) = canonical_str(&path) {
                map.entry(key).or_insert_with(Vec::new).push(WorkspaceRef {
                    id: id.into(),
                    label: label.into(),
                    kind: workspace_kind(label),
                    path: path.clone(),
                    tab_count,
                    pane_count,
                });
            }
            entries.push(Entry {
                source: Source::Workspace,
                title: label.into(),
                subtitle: format!("{} tabs:{} panes:{}", id, tab_count, pane_count),
                path,
                workspace_id: Some(id.into()),
                workspace_label: Some(label.into()),
                agent_target: None,
                project: None,
                action: EntryAction::FocusWorkspace { id: id.into() },
                source_label: None,
                search_terms: vec![id.into(), label.into()],
            });
        }
    }
    (entries, map)
}

fn workspace_kind(label: &str) -> WorkspaceKind {
    let label = label.trim().to_ascii_lowercase();
    if label.starts_with("project:") {
        WorkspaceKind::Project
    } else if label.starts_with("dir:") {
        WorkspaceKind::Dir
    } else {
        WorkspaceKind::Unknown
    }
}

pub(crate) fn collect_agents(
    workspaces: &[Entry],
    aliases: &[crate::config::AgentAliasConfig],
) -> Vec<Entry> {
    let workspace_labels: HashMap<&str, &str> = workspaces
        .iter()
        .filter_map(|entry| Some((entry.workspace_id.as_deref()?, entry.title.as_str())))
        .collect();
    let pane_json = herdr_json(["pane", "list"]).unwrap_or(Value::Null);
    let mut entries = Vec::new();
    if let Some(panes) = pane_json
        .pointer("/result/panes")
        .and_then(|v| v.as_array())
    {
        for p in panes {
            let Some(agent) = p.get("agent").and_then(|v| v.as_str()) else {
                continue;
            };
            let pane = p.get("pane_id").and_then(|v| v.as_str()).unwrap_or("");
            let tab = p.get("tab_id").and_then(|v| v.as_str()).unwrap_or("");
            let term = p
                .get("terminal_id")
                .and_then(|v| v.as_str())
                .unwrap_or(pane);
            let cwd = p.get("cwd").and_then(|v| v.as_str()).unwrap_or("/");
            let foreground_cwd = p
                .get("foreground_cwd")
                .and_then(|v| v.as_str())
                .unwrap_or(cwd);
            let status = p
                .get("agent_status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let workspace_id = p.get("workspace_id").and_then(|v| v.as_str()).unwrap_or("");
            let workspace_label = workspace_labels
                .get(workspace_id)
                .copied()
                .unwrap_or(workspace_id);
            let path = PathBuf::from(cwd);
            let dir = basename(&path);
            let alias_terms: Vec<String> = aliases
                .iter()
                .filter(|alias| alias.matches(agent, workspace_label, cwd))
                .map(|alias| alias.alias.clone())
                .collect();
            let title = format!("{agent} · {workspace_label} · {dir}");
            let subtitle = format!("{status} · {pane} · {tab}");
            let mut search_terms = vec![
                agent.into(),
                status.into(),
                pane.into(),
                tab.into(),
                term.into(),
                workspace_id.into(),
                workspace_label.into(),
                dir,
                basename(&PathBuf::from(foreground_cwd)),
                foreground_cwd.into(),
            ];
            search_terms.extend(alias_terms);
            entries.push(Entry {
                source: Source::Agent,
                title,
                subtitle,
                path,
                workspace_id: (!workspace_id.is_empty()).then(|| workspace_id.into()),
                workspace_label: Some(workspace_label.into()),
                agent_target: Some(term.into()),
                project: None,
                action: EntryAction::FocusAgent {
                    target: term.into(),
                },
                source_label: None,
                search_terms,
            });
        }
    }
    entries
}

pub(crate) fn collect_zoxide() -> Vec<Entry> {
    let Ok(out) = Command::new("zoxide").args(["query", "-l"]).output() else {
        return vec![];
    };
    if !out.status.success() {
        return vec![];
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let path = PathBuf::from(line);
            Entry {
                source: Source::Zoxide,
                title: basename(&path),
                subtitle: line.into(),
                path,
                workspace_id: None,
                workspace_label: None,
                agent_target: None,
                project: None,
                action: EntryAction::FocusOrCreateDir,
                source_label: None,
                search_terms: vec![],
            }
        })
        .collect()
}

pub(crate) fn collect_roots(config: &Config) -> Vec<Entry> {
    let mut out = Vec::new();
    for root in &config.roots {
        walk_dirs(&expand_path(&root.path), root.max_depth, &mut out);
    }
    out
}
fn walk_dirs(path: &Path, depth: usize, out: &mut Vec<Entry>) {
    if depth == 0 || !path.is_dir() {
        return;
    }
    if path.join(".git").exists()
        || path.join("package.json").exists()
        || path.join("Cargo.toml").exists()
    {
        out.push(Entry {
            source: Source::Root,
            title: basename(path),
            subtitle: path.display().to_string(),
            path: path.to_path_buf(),
            workspace_id: None,
            workspace_label: None,
            agent_target: None,
            project: None,
            action: EntryAction::FocusOrCreateDir,
            source_label: None,
            search_terms: vec![],
        });
    }
    if let Ok(read) = fs::read_dir(path) {
        for e in read.flatten() {
            let p = e.path();
            if p.is_dir() && !basename(&p).starts_with('.') {
                walk_dirs(&p, depth - 1, out);
            }
        }
    }
}
