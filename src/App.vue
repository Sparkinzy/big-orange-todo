<script setup lang="ts">
import { nextTick, onMounted, ref } from "vue"
import { listen } from "@tauri-apps/api/event"
import Panel from "./components/Panel.vue"
import Ball from "./components/Ball.vue"
import { api } from "./api"
import type { Todo } from "./types"

const todos = ref<Todo[]>([])
const mode = ref<"panel" | "ball">("ball")
const collapsed = ref(false)
const side = ref<"left" | "right">("right")
const fade = ref(false)
let busy = false

const sleep = (ms: number) => new Promise(r => setTimeout(r, ms))

/** 等 n 帧，确保 WebView 已按新窗口尺寸重排 */
const frames = (n: number) =>
  new Promise<void>(resolve => {
    let left = n
    const tick = () => (--left <= 0 ? resolve() : requestAnimationFrame(tick))
    requestAnimationFrame(tick)
  })

async function refresh() {
  todos.value = await api.list()
}

async function add(title: string, due: string | null) {
  await api.add(title, due)
  await refresh()
}
async function toggle(t: Todo) {
  await api.toggle(t.id, !t.done_at)
  await refresh()
}
async function remove(id: number) {
  await api.remove(id)
  await refresh()
}
async function openCard(id: number) {
  await api.openCard(id)
}

/** 窗口被 Rust 端 resize 后 WebView 不一定重绘，强制刷一帧 */
function repaint() {
  document.body.style.opacity = "0.999"
  requestAnimationFrame(() => {
    document.body.style.opacity = "1"
  })
}

/**
 * 窗口尺寸/位置的跳变无法在 Tauri 里做成平滑动画，
 * 所以把跳变夹在「淡出 → 换尺寸 → 淡入」之间，肉眼看不到硬切。
 */
async function swap(apply: () => Promise<void>) {
  fade.value = true
  await sleep(125)
  await apply()
  await sleep(45)
  fade.value = false
  repaint()
}

async function openPanel() {
  if (busy) return
  busy = true
  try {
    collapsed.value = false
    mode.value = "panel"
    await nextTick()
    await api.expand()
    repaint()
  } finally {
    busy = false
  }
}

async function toBall() {
  if (busy) return
  busy = true
  try {
    await swap(async () => {
      mode.value = "ball"
      collapsed.value = false
      await nextTick()
      await api.setMode("ball")
    })
  } finally {
    busy = false
  }
}

async function collapse() {
  if (busy || mode.value !== "panel" || collapsed.value) return
  busy = true
  try {
    collapsed.value = true
    await nextTick()
    await api.collapse()
    repaint()
  } finally {
    busy = false
  }
}

async function expand() {
  if (busy) return
  busy = true
  try {
    // 先把窗口撑到面板尺寸，等 WebView 完成重排，再插入面板
    // 否则面板会先在 96×117 的小窗口里被挤压一帧，看起来就是抖一下
    await api.expand()
    await frames(2)
    collapsed.value = false
    repaint()
  } finally {
    busy = false
  }
}

onMounted(async () => {
  await refresh()
  await listen<{ side: string }>("snap", e => {
    side.value = e.payload.side === "left" ? "left" : "right"
  })
  await listen<string>("cmd", async e => {
    if (e.payload === "panel") await openPanel()
    else if (e.payload === "ball") await toBall()
    else if (e.payload === "expand") await expand()
    else if (e.payload === "collapse") await collapse()
  })
  await listen("refresh", refresh)
})
</script>

<template>
  <div class="stage" :class="{ fade }">
    <Ball v-if="mode === 'ball'" :todos="todos" @open-panel="openPanel" />
    <Panel
      v-else
      :todos="todos"
      :side="side"
      :collapsed="collapsed"
      @expand="expand"
      @collapse="collapse"
      @add="add"
      @toggle="toggle"
      @remove="remove"
      @open-card="openCard"
      @to-ball="toBall"
    />
  </div>
</template>

<style scoped>
.stage {
  width: 100%;
  height: 100%;
  transition: opacity 125ms ease-out;
}
.stage.fade {
  opacity: 0;
}
</style>
