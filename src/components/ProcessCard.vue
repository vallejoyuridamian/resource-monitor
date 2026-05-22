<script setup lang="ts">
import { ref, computed } from 'vue'
import Sparkline from './Sparkline.vue'

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

interface History {
  ram: number[]
  cpu: number[]
}

const props = defineProps<{
  name: string
  stats?: ProcessStats
  history?: History
  color: string
}>()

const emit = defineEmits<{ remove: [name: string] }>()

const expanded = ref(false)
const scrollActive = ref(false)
let scrollTimer: ReturnType<typeof setTimeout> | null = null

function onSubScroll() {
  scrollActive.value = true
  if (scrollTimer) clearTimeout(scrollTimer)
  scrollTimer = setTimeout(() => { scrollActive.value = false }, 800)
}

const found    = computed(() => props.stats?.found ?? false)
const ramMb    = computed(() => props.stats?.ram_mb?.toFixed(1) ?? '—')
const cpu      = computed(() => props.stats?.cpu?.toFixed(2) ?? '—')
const pid      = computed(() => props.stats?.pid ?? '—')
const numCpus  = computed(() => props.stats?.num_cpus ?? null)
const children = computed(() => props.stats?.children ?? [])
const hasChildren = computed(() => found.value && children.value.length > 1)

const ramColor = computed(() => {
  if (!found.value) return 'var(--muted)'
  const mb = props.stats!.ram_mb
  if (mb > 400) return 'var(--red)'
  if (mb > 200) return 'var(--amber)'
  return props.color
})

const cpuColor = computed(() => {
  if (!found.value) return 'var(--muted)'
  const c = props.stats!.cpu
  if (c > 20) return 'var(--red)'
  if (c > 5)  return 'var(--amber)'
  return props.color
})

function indentPrefix(depth: number): string {
  if (depth === 0) return ''
  return '\u00a0'.repeat(depth - 1) + '└\u00a0'
}
</script>

<template>
  <div class="card" :class="{ dim: !found }">
    <div class="card-header">
      <div class="card-title">
        <span class="dot" :style="{ background: found ? color : 'var(--muted)' }" />
        <span class="name" :title="name">{{ name }}</span>
        <span class="pid">{{ found ? `PID ${pid}` : 'not found' }}</span>
      </div>
      <div class="header-actions">
        <button
          v-if="hasChildren"
          class="expand-btn"
          :title="expanded ? 'Collapse subprocess list' : 'Expand subprocess list'"
          @click="expanded = !expanded"
        >{{ expanded ? '▾' : '▸' }}</button>
        <button class="remove-btn" title="Remove" @click="emit('remove', name)">✕</button>
      </div>
    </div>

    <div class="metrics">
      <div class="metric">
        <span class="metric-label">RAM</span>
        <span class="metric-value" :style="{ color: ramColor }">
          {{ ramMb }}<span class="unit">MB</span>
        </span>
      </div>
      <div class="metric">
        <span class="metric-label">CPU<span v-if="numCpus" class="cpu-cores"> /{{ numCpus }}</span></span>
        <span class="metric-value" :style="{ color: cpuColor }">
          {{ cpu }}<span class="unit">%</span>
        </span>
      </div>
    </div>

    <div class="charts">
      <div class="chart-row">
        <span class="chart-label">RAM</span>
        <Sparkline :values="history?.ram ?? []" :color="color" />
      </div>
      <div class="chart-row">
        <span class="chart-label">CPU</span>
        <Sparkline :values="history?.cpu ?? []" :color="color" />
      </div>
    </div>

    <div v-if="expanded && hasChildren" class="subprocess-list">
      <div class="subprocess-header">
        <span>Process</span>
        <span>RAM</span>
        <span>CPU</span>
      </div>
      <div class="subprocess-scroll"
           :class="{ 'scroll-active': scrollActive }"
           @scroll="showScrollbar"
      >
        <div
          v-for="child in children"
          :key="child.pid"
          class="subprocess-row"
          :class="{ 'is-root': child.is_root }"
        >
          <span class="sub-name" :title="`${child.name} (PID ${child.pid})`">
              <span class="tree-indent">{{ indentPrefix(child.depth) }}</span><span class="sub-name-text">{{ child.name }}</span>
            </span>
          <span class="sub-ram" :style="{ color: child.ram_mb > 400 ? 'var(--red)' : child.ram_mb > 200 ? 'var(--amber)' : 'var(--muted)' }">
            {{ child.ram_mb.toFixed(1) }}<span class="sub-unit">MB</span>
          </span>
          <span class="sub-cpu">{{ child.cpu.toFixed(2) }}<span class="sub-unit">%</span></span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 16px;
  width: 280px;
  flex-shrink: 0;
  transition: border-color 0.15s, opacity 0.15s;
}
.card:not(.dim):hover { border-color: #444c56; }
.card.dim { opacity: 0.45; }

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 14px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 7px;
  min-width: 0;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
  transition: background 0.3s;
}

.name {
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 155px;
}

.pid {
  font-size: 10px;
  color: var(--muted);
  white-space: nowrap;
  flex-shrink: 0;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
}

.expand-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  font-size: 13px;
  padding: 3px 5px;
  border-radius: 4px;
  line-height: 1;
}
.expand-btn:hover { color: var(--text); background: rgba(255,255,255,0.06); }

.remove-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  font-size: 11px;
  padding: 3px 5px;
  border-radius: 4px;
  line-height: 1;
  flex-shrink: 0;
}
.remove-btn:hover { color: var(--text); background: rgba(255,255,255,0.06); }

.metrics {
  display: flex;
  gap: 24px;
  margin-bottom: 14px;
}

.metric { display: flex; flex-direction: column; gap: 3px; }

.metric-label {
  font-size: 10px;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.cpu-cores {
  opacity: 0.5;
  font-size: 9px;
  letter-spacing: 0;
  text-transform: none;
}

.metric-value {
  font-size: 28px;
  font-weight: 700;
  line-height: 1;
  transition: color 0.3s;
}

.unit {
  font-size: 12px;
  font-weight: 400;
  margin-left: 2px;
  color: var(--muted);
}

.charts { display: flex; flex-direction: column; gap: 6px; }

.chart-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.chart-label {
  font-size: 10px;
  color: var(--muted);
  width: 26px;
  flex-shrink: 0;
}

/* ── subprocess breakdown ───────────────────────── */
.subprocess-list {
  margin-top: 12px;
  border-top: 1px solid var(--border);
  padding-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.subprocess-header {
  display: grid;
  grid-template-columns: 1fr 62px 50px;
  gap: 8px;
  font-size: 9px;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  padding-bottom: 4px;
}

.subprocess-header span:nth-child(2),
.subprocess-header span:nth-child(3) {
  text-align: right;
}

.subprocess-scroll {
  max-height: 260px;
  overflow-y: auto;
  overflow-x: hidden;
}

.subprocess-scroll::-webkit-scrollbar { width: 3px; }
.subprocess-scroll::-webkit-scrollbar-track { background: transparent; }
.subprocess-scroll::-webkit-scrollbar-thumb {
  background: transparent;
  border-radius: 2px;
  transition: background 0.6s ease;
}
.subprocess-scroll:hover::-webkit-scrollbar-thumb,
.subprocess-scroll.scroll-active::-webkit-scrollbar-thumb {
  background: var(--border);
  transition: background 0s;
}

.subprocess-row {
  display: grid;
  grid-template-columns: 1fr 62px 50px;
  gap: 8px;
  align-items: baseline;
  font-size: 11px;
  color: var(--muted);
  padding: 1px 0;
}

.subprocess-row.is-root {
  color: var(--text);
  font-weight: 500;
}

.sub-name {
  display: flex;
  align-items: baseline;
  gap: 0;
  overflow: hidden;
  min-width: 0;
}

.sub-name-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.tree-indent {
  opacity: 0.25;
  font-size: 9px;
  flex-shrink: 0;
  white-space: pre;
}

.sub-ram, .sub-cpu {
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
  text-align: right;
}

.sub-unit {
  font-size: 9px;
  margin-left: 1px;
  opacity: 0.6;
}
</style>
