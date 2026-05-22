<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import ProcessCard from './components/ProcessCard.vue'
import SnapshotTable from './components/SnapshotTable.vue'
import ProcessPicker from './components/ProcessPicker.vue'

const HISTORY_LEN = 60 // 120s at 2000ms
const COLORS = ['#3fb950', '#58a6ff', '#f78166', '#d2a8ff', '#ffa657', '#79c0ff']

interface SubprocessEntry {
  name: string
  pid: number
  ram_mb: number
  cpu: number
  is_root: boolean
  depth: number
}

interface ProcessStats {
  name: string
  pid: number
  cpu: number
  num_cpus: number
  ram_mb: number
  found: boolean
  children: SubprocessEntry[]
}

interface ProcessEntry {
  name: string
  color: string
}

interface SnapshotEntry {
  name: string
  ramMb: number
  cpu: number
}

interface Snapshot {
  label: string
  time: string
  entries: SnapshotEntry[]
}

const processes = ref<ProcessEntry[]>([
  { name: 'universal-clipboard', color: COLORS[0] },
])

const snapshotLabel = ref('')
const currentStats = reactive<Record<string, ProcessStats>>({})
const histories = reactive<Record<string, { ram: number[]; cpu: number[] }>>({})
const snapshots = ref<Snapshot[]>([])

let intervalId: ReturnType<typeof setInterval> | null = null

function ensureHistory(name: string) {
  if (!histories[name]) histories[name] = { ram: [], cpu: [] }
}

async function tick() {
  const names = processes.value.map(p => p.name)
  if (!names.length) return
  try {
    const stats = await invoke<ProcessStats[]>('get_stats', { names })
    for (const s of stats) {
      currentStats[s.name] = s
      ensureHistory(s.name)
      const h = histories[s.name]
      h.ram.push(s.found ? s.ram_mb : 0)
      h.cpu.push(s.found ? s.cpu : 0)
      if (h.ram.length > HISTORY_LEN) h.ram.shift()
      if (h.cpu.length > HISTORY_LEN) h.cpu.shift()
    }
  } catch (e) {
    console.error('get_stats failed:', e)
  }
}

function addProcess(name: string) {
  if (!name || processes.value.some(p => p.name === name)) return
  const color = COLORS[processes.value.length % COLORS.length]
  processes.value.push({ name, color })
  ensureHistory(name)
}

function removeProcess(name: string) {
  processes.value = processes.value.filter(p => p.name !== name)
  delete currentStats[name]
  delete histories[name]
}

function takeSnapshot() {
  const label = snapshotLabel.value.trim() || `Snapshot ${snapshots.value.length + 1}`
  snapshots.value.push({
    label,
    time: new Date().toLocaleTimeString(),
    entries: processes.value.map(p => ({
      name: p.name,
      ramMb: currentStats[p.name]?.ram_mb ?? 0,
      cpu: currentStats[p.name]?.cpu ?? 0,
    })),
  })
  snapshotLabel.value = ''
}

function exportCsv() {
  if (!snapshots.value.length) return
  const allNames = [...new Set(snapshots.value.flatMap(s => s.entries.map(e => e.name)))]
  const header = ['snapshot', 'time', ...allNames.flatMap(n => [`${n}_ram_mb`, `${n}_cpu`])]
  const rows = snapshots.value.map(s => {
    const byName = Object.fromEntries(s.entries.map(e => [e.name, e]))
    return [
      `"${s.label}"`,
      s.time,
      ...allNames.flatMap(n => [
        byName[n]?.ramMb.toFixed(1) ?? '',
        byName[n]?.cpu.toFixed(2) ?? '',
      ]),
    ]
  })
  const csv = [header, ...rows].map(r => r.join(',')).join('\n')
  const a = document.createElement('a')
  a.href = 'data:text/csv;charset=utf-8,' + encodeURIComponent(csv)
  a.download = `resource-monitor-${Date.now()}.csv`
  a.click()
}

onMounted(() => {
  processes.value.forEach(p => ensureHistory(p.name))
  tick()
  intervalId = setInterval(tick, 2000)
})

onUnmounted(() => {
  if (intervalId) clearInterval(intervalId)
})
</script>

<template>
  <div class="app">
    <header class="header">
      <div class="header-left">
        <span class="logo">⬡</span>
        <h1>Resource Monitor</h1>
        <span class="pulse" />
      </div>
      <div class="controls">
        <ProcessPicker :existing="processes.map(p => p.name)" @add="addProcess" />
        <div class="sep" />
        <input
          v-model="snapshotLabel"
          class="input"
          placeholder="Snapshot label…"
          @keyup.enter="takeSnapshot"
        />
        <button class="btn btn-accent" @click="takeSnapshot">📸 Snapshot</button>
        <button class="btn btn-ghost" :disabled="!snapshots.length" @click="exportCsv">
          Export CSV
        </button>
      </div>
    </header>

    <div class="process-grid">
      <ProcessCard
        v-for="proc in processes"
        :key="proc.name"
        :name="proc.name"
        :stats="currentStats[proc.name]"
        :history="histories[proc.name]"
        :color="proc.color"
        @remove="removeProcess(proc.name)"
      />
      <div v-if="!processes.length" class="empty">
        Add a process name above to start monitoring.
      </div>
    </div>

    <SnapshotTable
      v-if="snapshots.length"
      :snapshots="snapshots"
      :process-names="processes.map(p => p.name)"
    />
  </div>
</template>

<style>
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

:root {
  --bg:     #0d1117;
  --card:   #161b22;
  --border: #30363d;
  --text:   #e6edf3;
  --muted:  #8b949e;
  --green:  #3fb950;
  --blue:   #58a6ff;
  --amber:  #d29922;
  --red:    #f85149;
  font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', ui-monospace, monospace;
}

html, body, #app { height: 100%; }

.app {
  background: var(--bg);
  color: var(--text);
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

/* ── Header ─────────────────────────────────────── */
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 20px;
  background: #010409;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  gap: 16px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

.logo { font-size: 18px; color: var(--green); }

h1 {
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--muted);
}

.pulse {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--green);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.25; }
}

.controls {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.input {
  background: var(--card);
  border: 1px solid var(--border);
  color: var(--text);
  padding: 5px 10px;
  border-radius: 6px;
  font-size: 12px;
  font-family: inherit;
  outline: none;
  width: 150px;
  transition: border-color 0.15s;
}
.input:focus { border-color: var(--green); }
.input::placeholder { color: var(--muted); }

.btn {
  padding: 5px 12px;
  border-radius: 6px;
  border: 1px solid var(--border);
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}
.btn-ghost { background: transparent; color: var(--muted); }
.btn-ghost:hover { background: var(--card); color: var(--text); }
.btn-ghost:disabled { opacity: 0.3; cursor: default; }

.btn-accent {
  background: rgba(63,185,80,0.12);
  color: var(--green);
  border-color: rgba(63,185,80,0.3);
}
.btn-accent:hover { background: rgba(63,185,80,0.22); }

.sep { width: 1px; height: 22px; background: var(--border); }

/* ── Grid ────────────────────────────────────────── */
.process-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  padding: 20px;
  flex: 1;
  align-content: flex-start;
  align-items: flex-start;
}

.empty {
  color: var(--muted);
  font-size: 13px;
  padding: 48px;
  width: 100%;
  text-align: center;
}
</style>
