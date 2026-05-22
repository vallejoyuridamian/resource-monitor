<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface ProcessSummary {
  name: string
  pid: number
  ram_mb: number
}

const emit = defineEmits<{ add: [name: string] }>()
const props = defineProps<{ existing: string[] }>()

const open = ref(false)
const filter = ref('')
const processes = ref<ProcessSummary[]>([])
const loading = ref(false)
const inputRef = ref<HTMLInputElement | null>(null)
const rootRef = ref<HTMLDivElement | null>(null)

interface GroupedProcess {
  name: string
  count: number
  ram_mb: number
}

const grouped = computed<GroupedProcess[]>(() => {
  const map = new Map<string, GroupedProcess>()
  for (const p of processes.value) {
    const existing = map.get(p.name)
    if (existing) {
      existing.count++
      existing.ram_mb += p.ram_mb
    } else {
      map.set(p.name, { name: p.name, count: 1, ram_mb: p.ram_mb })
    }
  }
  return [...map.values()].sort((a, b) => b.ram_mb - a.ram_mb)
})

const filtered = computed(() => {
  const q = filter.value.toLowerCase()
  return grouped.value.filter(p =>
    !props.existing.includes(p.name) &&
    (q === '' || p.name.toLowerCase().includes(q))
  )
})

let refreshId: ReturnType<typeof setInterval> | null = null

async function fetchProcesses() {
  try {
    processes.value = await invoke<ProcessSummary[]>('list_processes')
  } finally {
    loading.value = false
  }
}

async function openPicker() {
  open.value = true
  loading.value = true
  filter.value = ''
  await nextTick()
  inputRef.value?.focus()
  await fetchProcesses()
  refreshId = setInterval(fetchProcesses, 5000)
}

function closePicker() {
  open.value = false
  filter.value = ''
  if (refreshId) { clearInterval(refreshId); refreshId = null }
}

function select(name: string) {
  emit('add', name)
  closePicker()
}

function onClickOutside(e: MouseEvent) {
  if (rootRef.value && !rootRef.value.contains(e.target as Node)) {
    closePicker()
  }
}

onMounted(() => document.addEventListener('mousedown', onClickOutside))
onUnmounted(() => { document.removeEventListener('mousedown', onClickOutside); if (refreshId) clearInterval(refreshId) })
</script>

<template>
  <div ref="rootRef" class="picker-root">
    <button class="input picker-trigger" @click="openPicker">
      {{ open ? '' : 'Process name…' }}
    </button>

    <div v-if="open" class="picker-dropdown">
      <input
        ref="inputRef"
        v-model="filter"
        class="picker-search"
        placeholder="Filter…"
      />
      <div class="picker-list">
        <div v-if="loading" class="picker-msg">Loading…</div>
        <div v-else-if="!filtered.length" class="picker-msg">No matches</div>
        <button
          v-for="p in filtered"
          :key="p.name"
          class="picker-item"
          @mousedown.prevent="select(p.name)"
        >
          <span class="picker-item-name">{{ p.name }}</span>
          <span class="picker-item-meta">
            <span v-if="p.count > 1" class="picker-item-count">×{{ p.count }}</span>
            {{ p.ram_mb.toFixed(0) }} MB
          </span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.picker-root {
  position: relative;
}

.picker-trigger {
  cursor: pointer;
  text-align: left;
  min-width: 160px;
  color: var(--text-muted);
}

.picker-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  width: 280px;
  background: #161b22;
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
  z-index: 100;
  overflow: hidden;
}

.picker-search {
  width: 100%;
  box-sizing: border-box;
  background: #161b22;
  border: none;
  border-bottom: 1px solid var(--border);
  border-radius: 0;
  padding: 8px 12px;
  color: var(--text);
  font-size: 13px;
  outline: none;
}

.picker-list {
  max-height: 280px;
  overflow-y: auto;
}

.picker-msg {
  padding: 12px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}

.picker-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  padding: 8px 12px;
  background: none;
  border: none;
  border-radius: 0;
  color: var(--text);
  font-size: 13px;
  cursor: pointer;
  text-align: left;
  gap: 8px;
}

.picker-item:hover {
  background: rgba(255, 255, 255, 0.06);
}

.picker-item-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.picker-item-meta {
  color: var(--text-muted);
  font-size: 12px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 4px;
}

.picker-item-count {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  padding: 1px 5px;
  font-size: 11px;
}
</style>
