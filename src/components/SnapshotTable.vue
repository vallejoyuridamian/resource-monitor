<script setup lang="ts">
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

defineProps<{
  snapshots: Snapshot[]
  processNames: string[]
}>()
</script>

<template>
  <div class="section">
    <p class="section-title">Snapshots</p>
    <div class="wrap">
      <table>
        <thead>
          <tr>
            <th>Label</th>
            <th>Time</th>
            <th v-for="n in processNames" :key="n" colspan="2">{{ n }}</th>
          </tr>
          <tr class="sub">
            <th colspan="2" />
            <template v-for="n in processNames" :key="n">
              <th>RAM MB</th>
              <th>CPU %</th>
            </template>
          </tr>
        </thead>
        <tbody>
          <tr v-for="snap in snapshots" :key="snap.label + snap.time">
            <td class="bold">{{ snap.label }}</td>
            <td class="dim">{{ snap.time }}</td>
            <template v-for="n in processNames" :key="n">
              <td class="num">{{ snap.entries.find(e => e.name === n)?.ramMb.toFixed(1) ?? '—' }}</td>
              <td class="num">{{ snap.entries.find(e => e.name === n)?.cpu.toFixed(2) ?? '—' }}</td>
            </template>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.section {
  border-top: 1px solid var(--border);
  padding: 14px 20px 18px;
  flex-shrink: 0;
}

.section-title {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--muted);
  font-weight: 600;
  margin-bottom: 10px;
}

.wrap { overflow-x: auto; }

table {
  border-collapse: collapse;
  width: 100%;
  font-size: 12px;
}

th {
  color: var(--muted);
  text-align: left;
  padding: 4px 14px;
  font-weight: 500;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}

tr.sub th { font-size: 10px; }

td {
  padding: 6px 14px;
  border-bottom: 1px solid rgba(48,54,61,0.5);
  white-space: nowrap;
}

td.bold { font-weight: 600; }
td.dim  { color: var(--muted); }
td.num  { font-variant-numeric: tabular-nums; color: var(--green); }
</style>
