<script setup lang="ts">
/**
 * ChainVisualization.vue
 *
 * SVG-rendered OCEL lifecycle flow diagram + chain integrity summary.
 * Ported from dashboard.bak/app/components/evidence/ChainVerifier.vue and
 * dashboard.bak/app/components/ml/ProcessMiningVisualization.vue,
 * adapted to rocket-craft's monospace dark-terminal style.
 *
 * Shows:
 * - Cook pipeline OCEL activities as connected nodes (CookStarted → … → PackageVerified)
 * - Chain integrity status (PASS/FAIL/UNKNOWN + break count)
 * - Sessions checked count
 *
 * Props:
 *   lifecycle   — ordered activity names (from a game_receipt row)
 *   chainStatus — result of GET /api/game/chain-verify
 */

interface ChainStatus {
  overall: 'PASS' | 'FAIL' | 'UNKNOWN';
  sessions_checked: number;
  breaks: Array<{ session_id: string; message: string; broken_at: number | null }>;
}

const props = withDefaults(defineProps<{
  lifecycle?: string[];
  chainStatus?: ChainStatus | null;
  title?: string;
}>(), {
  lifecycle: () => ['CookStarted', 'WasmPackaged', 'JsEmitted', 'DataPakStaged', 'PackageVerified'],
  chainStatus: null,
  title: 'OCEL Pipeline Flow',
});

// ── Node layout ──────────────────────────────────────────────────────────────
const NODE_W = 110;
const NODE_H = 32;
const NODE_GAP = 18;
const ROW_H = 60;
const MAX_PER_ROW = 4;
const PAD = 16;

interface FlowNode {
  label: string;
  x: number;
  y: number;
  isError: boolean;
  isComplete: boolean;
}

const nodes = computed<FlowNode[]>(() => {
  return props.lifecycle.map((activity, i) => {
    const row = Math.floor(i / MAX_PER_ROW);
    const col = i % MAX_PER_ROW;
    // Alternate row direction (snake layout)
    const colActual = row % 2 === 0 ? col : (MAX_PER_ROW - 1 - col);
    return {
      label: activity.replace(/([A-Z])/g, ' $1').trim(),
      x: PAD + colActual * (NODE_W + NODE_GAP),
      y: PAD + row * ROW_H,
      isError: activity.includes('Error') || activity.includes('Failed'),
      isComplete: activity === 'PackageVerified' || activity === 'CookFinished',
    };
  });
});

const svgWidth = computed(() =>
  PAD * 2 + Math.min(props.lifecycle.length, MAX_PER_ROW) * (NODE_W + NODE_GAP) - NODE_GAP
);
const svgHeight = computed(() =>
  PAD * 2 + (Math.ceil(props.lifecycle.length / MAX_PER_ROW)) * ROW_H
);

function nodeColor(node: FlowNode): string {
  if (node.isError) return '#7f1d1d';
  if (node.isComplete) return '#14532d';
  return '#1e3a5f';
}
function nodeBorder(node: FlowNode): string {
  if (node.isError) return '#ef4444';
  if (node.isComplete) return '#22c55e';
  return '#3b82f6';
}

// Edges: connect consecutive nodes with lines
interface Edge { x1: number; y1: number; x2: number; y2: number; }
const edges = computed<Edge[]>(() => {
  const result: Edge[] = [];
  for (let i = 0; i < nodes.value.length - 1; i++) {
    const a = nodes.value[i]!;
    const b = nodes.value[i + 1]!;
    const row_a = Math.floor(i / MAX_PER_ROW);
    const row_b = Math.floor((i + 1) / MAX_PER_ROW);
    if (row_a === row_b) {
      // Same row — horizontal connector from right edge of a to left edge of b
      result.push({
        x1: a.x + NODE_W, y1: a.y + NODE_H / 2,
        x2: b.x, y2: b.y + NODE_H / 2,
      });
    } else {
      // Row break — vertical drop then horizontal
      result.push({
        x1: a.x + NODE_W / 2, y1: a.y + NODE_H,
        x2: b.x + NODE_W / 2, y2: b.y,
      });
    }
  }
  return result;
});

// ── Chain integrity summary ────────────────────────────────────────────────
const chainColor = computed(() => {
  if (!props.chainStatus) return '#475569';
  return props.chainStatus.overall === 'PASS' ? '#22c55e'
       : props.chainStatus.overall === 'FAIL' ? '#ef4444'
       : '#94a3b8';
});
const integrityPct = computed(() => {
  if (!props.chainStatus || props.chainStatus.sessions_checked === 0) return null;
  const good = props.chainStatus.sessions_checked - props.chainStatus.breaks.length;
  return Math.round((good / props.chainStatus.sessions_checked) * 100);
});
</script>

<template>
  <div class="chain-viz">
    <div class="chain-viz-header">
      <span class="chain-viz-title">{{ title }}</span>
      <span v-if="chainStatus" class="chain-integrity-badge" :style="{ color: chainColor, borderColor: chainColor }">
        {{ chainStatus.overall }}
        <span v-if="integrityPct !== null"> · {{ integrityPct }}%</span>
      </span>
    </div>

    <!-- SVG process flow -->
    <svg
      class="flow-svg"
      :width="svgWidth"
      :height="svgHeight"
      :viewBox="`0 0 ${svgWidth} ${svgHeight}`"
    >
      <defs>
        <marker id="arrow" markerWidth="6" markerHeight="5" refX="6" refY="2.5" orient="auto">
          <polygon points="0 0, 6 2.5, 0 5" fill="#334155" />
        </marker>
      </defs>

      <!-- Edges -->
      <line
        v-for="(e, i) in edges"
        :key="`e-${i}`"
        :x1="e.x1" :y1="e.y1" :x2="e.x2" :y2="e.y2"
        stroke="#334155"
        stroke-width="1.5"
        marker-end="url(#arrow)"
      />

      <!-- Nodes -->
      <g v-for="(n, i) in nodes" :key="`n-${i}`">
        <rect
          :x="n.x" :y="n.y"
          :width="NODE_W" :height="NODE_H"
          rx="4"
          :fill="nodeColor(n)"
          :stroke="nodeBorder(n)"
          stroke-width="1.5"
        />
        <text
          :x="n.x + NODE_W / 2"
          :y="n.y + NODE_H / 2 + 1"
          text-anchor="middle"
          dominant-baseline="middle"
          font-family="'Courier New', monospace"
          font-size="9"
          fill="#e2e8f0"
        >{{ n.label }}</text>
      </g>
    </svg>

    <!-- Chain break list -->
    <div v-if="chainStatus?.breaks.length" class="break-list">
      <div v-for="b in chainStatus.breaks" :key="b.session_id" class="break-item">
        <span class="break-id">{{ b.session_id.slice(0, 8) }}…</span>
        <span class="break-msg">{{ b.message }}</span>
        <span v-if="b.broken_at !== null" class="break-seq">@seq{{ b.broken_at }}</span>
      </div>
    </div>

    <div v-if="chainStatus?.sessions_checked" class="chain-meta">
      {{ chainStatus.sessions_checked }} session(s) checked
      · {{ chainStatus.breaks.length }} break(s)
    </div>
  </div>
</template>

<style scoped>
.chain-viz {
  background: #0b0f19;
  border: 1px solid #1e293b;
  border-radius: 6px;
  padding: 1rem;
  font-family: 'Courier New', monospace;
}
.chain-viz-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.75rem;
}
.chain-viz-title { font-size: 0.8rem; color: #94a3b8; font-weight: 600; }
.chain-integrity-badge {
  font-size: 0.72rem;
  border: 1px solid;
  padding: 0.1rem 0.5rem;
  border-radius: 999px;
  font-weight: 700;
}
.flow-svg { display: block; max-width: 100%; overflow: visible; }
.break-list { margin-top: 0.75rem; display: flex; flex-direction: column; gap: 0.25rem; }
.break-item {
  display: flex; align-items: baseline; gap: 0.5rem;
  font-size: 0.7rem; color: #fca5a5;
  border-left: 2px solid #ef4444; padding-left: 0.4rem;
}
.break-id { font-weight: 700; color: #f87171; }
.break-msg { flex: 1; color: #fca5a5; }
.break-seq { color: #f97316; }
.chain-meta { font-size: 0.65rem; color: #334155; margin-top: 0.5rem; }
</style>
