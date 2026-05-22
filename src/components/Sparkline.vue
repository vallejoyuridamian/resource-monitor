<script setup lang="ts">
import { computed } from 'vue'

interface Point { x: number; y: number }

const props = defineProps<{
  values: number[]
  color: string
  width?: number
  height?: number
}>()

const W = computed(() => props.width ?? 220)
const H = computed(() => props.height ?? 36)

const pts = computed<Point[]>(() => {
  const v = props.values
  if (v.length < 2) return []
  const min = Math.min(...v)
  const max = Math.max(...v)
  const range = max - min || 1
  const step = W.value / (v.length - 1)
  return v.map((val, i) => ({
    x: i * step,
    y: H.value - 2 - ((val - min) / range) * (H.value - 4),
  }))
})

const linePts = computed(() =>
  pts.value.map(p => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' ')
)

const fillPts = computed(() => {
  if (!pts.value.length) return ''
  const last  = pts.value[pts.value.length - 1]
  const first = pts.value[0]
  return `${linePts.value} ${last.x.toFixed(1)},${H.value} ${first.x.toFixed(1)},${H.value}`
})

const lastPt  = computed(() => pts.value[pts.value.length - 1] ?? null)
const gradId  = computed(() => `g${props.color.replace('#', '')}`)
</script>

<template>
  <svg :width="W" :height="H" class="spark" overflow="visible">
    <defs>
      <linearGradient :id="gradId" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%"   :stop-color="color" stop-opacity="0.28" />
        <stop offset="100%" :stop-color="color" stop-opacity="0.02" />
      </linearGradient>
    </defs>

    <template v-if="pts.length >= 2">
      <polygon :points="fillPts" :fill="`url(#${gradId})`" />
      <polyline
        :points="linePts"
        :stroke="color"
        stroke-width="1.5"
        fill="none"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <circle v-if="lastPt" :cx="lastPt.x" :cy="lastPt.y" r="2.5" :fill="color" />
    </template>

    <!-- placeholder when no data yet -->
    <line
      v-else
      x1="0" :y1="H / 2" :x2="W" :y2="H / 2"
      stroke="var(--border)"
      stroke-width="1"
      stroke-dasharray="4,4"
    />
  </svg>
</template>

<style scoped>
.spark { display: block; flex: 1; }
</style>
