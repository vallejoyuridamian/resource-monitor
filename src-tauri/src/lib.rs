use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tauri::State;

/// Per-process metadata cached between poll ticks.
/// Reading /proc/[pid]/status is expensive at scale; we cache and only
/// re-read for PIDs we have not seen before.
struct ProcEntry {
    /// True when this PID is a thread-group leader (a real process, not a thread).
    is_leader: bool,
    /// Parent PID from the PPid: field — used to build the process tree.
    ppid: u32,
}

pub struct AppState {
    system: Mutex<System>,
    /// pid → ProcEntry; built incrementally, one /proc read per new PID.
    proc_info: Mutex<HashMap<u32, ProcEntry>>,
    /// Number of logical CPUs, read once at startup from sysinfo.
    num_cpus: usize,
}

#[derive(serde::Serialize, Clone)]
pub struct ProcessStats {
    name: String,
    pid: u32,
    /// CPU as % of total machine capacity (100 = all cores fully used).
    cpu: f32,
    /// Number of logical CPUs, so the frontend can show context.
    num_cpus: usize,
    ram_mb: f64,
    found: bool,
    /// Per-process breakdown (sorted by RAM desc) for the expanded view.
    children: Vec<SubprocessEntry>,
}

#[derive(serde::Serialize, Clone)]
pub struct ProcessSummary {
    name: String,
    pid: u32,
    ram_mb: f64,
}

/// One entry in the subprocess breakdown shown in the card's expanded view.
#[derive(serde::Serialize, Clone)]
pub struct SubprocessEntry {
    name: String,
    pid: u32,
    ram_mb: f64,
    cpu: f32,
    /// True when this PID was a name-match root (depth == 0).
    is_root: bool,
    /// Tree depth from the matched root (0 = root, 1 = direct child, …).
    depth: u32,
}

/// Reads Tgid: and PPid: from /proc/[pid]/status in a single pass.
/// Only called for PIDs not yet in the cache — never repeated for the same PID.
#[cfg(target_os = "linux")]
fn read_proc_status(pid: u32) -> ProcEntry {
    let path = format!("/proc/{}/status", pid);
    let mut tgid = pid;
    let mut ppid = 0u32;
    if let Ok(contents) = std::fs::read_to_string(&path) {
        for line in contents.lines() {
            if let Some(rest) = line.strip_prefix("Tgid:") {
                if let Ok(v) = rest.trim().parse::<u32>() { tgid = v; }
            } else if let Some(rest) = line.strip_prefix("PPid:") {
                if let Ok(v) = rest.trim().parse::<u32>() { ppid = v; }
            }
        }
    }
    ProcEntry { is_leader: tgid == pid, ppid }
}

#[cfg(not(target_os = "linux"))]
fn read_proc_status(pid: u32) -> ProcEntry {
    ProcEntry { is_leader: true, ppid: 0 }
}

/// Synchronises proc_info with the current sysinfo snapshot.
/// Removes dead PIDs; adds new ones with one /proc read each (never repeated).
fn sync_proc_info(sys: &System, info: &mut HashMap<u32, ProcEntry>) {
    let current: HashSet<u32> = sys.processes().keys().map(|p| p.as_u32()).collect();
    info.retain(|pid, _| current.contains(pid));
    for &pid in &current {
        if !info.contains_key(&pid) {
            info.insert(pid, read_proc_status(pid));
        }
    }
}

/// DFS from `root_pids`, returning (pid, depth) in tree order.
/// Children at each level are sorted by RAM desc so the heaviest subtrees appear first.
/// Depth 0 = matched root, 1 = direct child, 2 = grandchild, …
fn collect_subtree_dfs(
    root_pids: &[u32],
    info: &HashMap<u32, ProcEntry>,
    ram_map: &HashMap<u32, u64>,
) -> Vec<(u32, u32)> {
    let mut ch_map: HashMap<u32, Vec<u32>> = HashMap::new();
    for (&pid, entry) in info {
        if entry.is_leader {
            ch_map.entry(entry.ppid).or_default().push(pid);
        }
    }
    // Pre-sort each node's children by RAM desc so DFS visits heavy branches first.
    for kids in ch_map.values_mut() {
        kids.sort_by(|a, b| ram_map.get(b).unwrap_or(&0).cmp(ram_map.get(a).unwrap_or(&0)));
    }

    let mut result = Vec::new();
    let mut visited: HashSet<u32> = HashSet::new();
    // Stack: push roots in reverse so first root is popped (processed) first.
    let mut stack: Vec<(u32, u32)> = root_pids.iter().rev()
        .filter_map(|&pid| if visited.insert(pid) { Some((pid, 0u32)) } else { None })
        .collect();

    while let Some((pid, depth)) = stack.pop() {
        result.push((pid, depth));
        if let Some(kids) = ch_map.get(&pid) {
            // Push in reverse so the highest-RAM child is popped (visited) first.
            for &kid in kids.iter().rev() {
                if visited.insert(kid) {
                    stack.push((kid, depth + 1));
                }
            }
        }
    }
    result
}

/// Sample CPU + RAM for a list of process name substrings.
/// Threads are deduplicated via TGID. Child processes (e.g. WebKitWebProcess
/// spawned by a Tauri binary) are included via process-tree BFS so that each
/// card shows the true total cost of that application.
#[tauri::command]
fn get_stats(names: Vec<String>, state: State<'_, AppState>) -> Vec<ProcessStats> {
    let mut sys = state.system.lock().unwrap();
    sys.refresh_processes();

    let mut info = state.proc_info.lock().unwrap();
    sync_proc_info(&sys, &mut info);

    names
        .iter()
        .map(|search| {
            let needle = search.to_lowercase();

            // Find thread-group leaders whose name or exe basename matches.
            let root_pids: Vec<u32> = sys
                .processes()
                .values()
                .filter(|p| {
                    let pid = p.pid().as_u32();
                    if !info.get(&pid).map(|e| e.is_leader).unwrap_or(true) {
                        return false; // skip threads
                    }
                    let exe_basename = p
                        .exe()
                        .and_then(|e| e.file_name())
                        .map(|n| n.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    let name = p.name().to_lowercase();
                    name.contains(&needle) || exe_basename.contains(&needle)
                })
                .map(|p| p.pid().as_u32())
                .collect();

            if root_pids.is_empty() {
                return ProcessStats {
                    name: search.clone(),
                    pid: 0,
                    cpu: 0.0,
                    num_cpus: state.num_cpus,
                    ram_mb: 0.0,
                    found: false,
                    children: vec![],
                };
            }

            // Build a RAM snapshot used only for ordering the DFS traversal.
            let ram_map: HashMap<u32, u64> = sys.processes()
                .iter()
                .map(|(pid, p)| (pid.as_u32(), p.memory()))
                .collect();

            // DFS walk: returns (pid, depth) in tree order, heavy branches first.
            let subtree = collect_subtree_dfs(&root_pids, &info, &ram_map);

            let children: Vec<SubprocessEntry> = subtree.iter()
                .filter_map(|&(pid, depth)| {
                    sys.process(sysinfo::Pid::from(pid as usize)).map(|p| {
                        let name = p.exe().and_then(|e| e.file_name())
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| p.name().to_string());
                        SubprocessEntry {
                            name,
                            pid,
                            ram_mb: p.memory() as f64 / (1024.0 * 1024.0),
                            cpu: p.cpu_usage() / state.num_cpus as f32,
                            is_root: depth == 0,
                            depth,
                        }
                    })
                })
                .collect();
            // Order comes from DFS — no extra sort needed.

            let total_ram: f64 = children.iter().map(|c| c.ram_mb).sum();
            let cpu: f32 = children.iter().map(|c| c.cpu).sum();

            ProcessStats {
                name: search.clone(),
                pid: root_pids[0],
                cpu,
                num_cpus: state.num_cpus,
                ram_mb: total_ram,
                found: true,
                children,
            }
        })
        .collect()
}

/// Returns all running processes (thread-group leaders only), sorted by RAM descending.
#[tauri::command]
fn list_processes(state: State<'_, AppState>) -> Vec<ProcessSummary> {
    let mut sys = state.system.lock().unwrap();
    sys.refresh_processes();

    let mut info = state.proc_info.lock().unwrap();
    sync_proc_info(&sys, &mut info);

    let mut result: Vec<ProcessSummary> = sys
        .processes()
        .values()
        .filter(|p| {
            let pid = p.pid().as_u32();
            info.get(&pid).map(|e| e.is_leader).unwrap_or(true)
        })
        .map(|p| {
            let name = p
                .exe()
                .and_then(|e| e.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| p.name().to_string());
            ProcessSummary {
                name,
                pid: p.pid().as_u32(),
                ram_mb: p.memory() as f64 / (1024.0 * 1024.0),
            }
        })
        .collect();

    result.sort_by(|a, b| b.ram_mb.partial_cmp(&a.ram_mb).unwrap_or(std::cmp::Ordering::Equal));
    result
}

pub fn run() {
    // One-time CPU refresh to read the hardware topology.
    let sys = System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::new()),
    );
    let num_cpus = sys.cpus().len().max(1);
    // Reset to a plain System for subsequent process refreshes.
    let sys = System::new();

    tauri::Builder::default()
        .manage(AppState {
            system: Mutex::new(sys),
            proc_info: Mutex::new(HashMap::new()),
            num_cpus,
        })
        .invoke_handler(tauri::generate_handler![get_stats, list_processes])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
