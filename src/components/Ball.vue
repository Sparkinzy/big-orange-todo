<script setup lang="ts">
import { computed } from "vue"
import { getCurrentWindow } from "@tauri-apps/api/window"
import type { Todo } from "../types"
import { daysLeft } from "../date"
import badgeCat0 from "../assets/cat/badge-cat-0.png"
import badgeCat1 from "../assets/cat/badge-cat-1.png"
import badgeCat2 from "../assets/cat/badge-cat-2.png"
import badgeCat3 from "../assets/cat/badge-cat-3.png"
import badgeCat4 from "../assets/cat/badge-cat-4.png"
import badgeCat5 from "../assets/cat/badge-cat-5.png"

const props = defineProps<{ todos: Todo[] }>()
const emit = defineEmits<{ (e: "open-panel"): void }>()

const open = computed(() => props.todos.filter(t => !t.done_at))
const overdue = computed(() => open.value.filter(t => (daysLeft(t.due_at) ?? 1) < 0).length)
const badge = computed(() => (overdue.value > 0 ? overdue.value : open.value.length))

// 线团档位按未完成总数走：0 窝着睡 / 1-4 对应线团数 / 5 以上忙昏头
const BADGE_CATS = [badgeCat0, badgeCat1, badgeCat2, badgeCat3, badgeCat4, badgeCat5]
const badgeImg = computed(() => BADGE_CATS[Math.min(open.value.length, 5)])

let down = false
let moved = false
let sx = 0
let sy = 0

function onDown(e: MouseEvent) {
  down = true
  moved = false
  sx = e.screenX
  sy = e.screenY
}

function onMove(e: MouseEvent) {
  if (!down || moved) return
  if (Math.abs(e.screenX - sx) > 4 || Math.abs(e.screenY - sy) > 4) {
    moved = true
    down = false
    getCurrentWindow().startDragging()
  }
}

function onUp() {
  if (down && !moved) emit("open-panel")
  down = false
}
</script>

<template>
  <div class="ball" @mousedown="onDown" @mousemove="onMove" @mouseup="onUp">
    <img class="core" :src="badgeImg" alt="" draggable="false" />
    <span v-if="badge > 0" class="badge" :class="{ warn: overdue > 0 }">{{ badge > 99 ? "99+" : badge }}</span>
  </div>
</template>

<style scoped>
.ball {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.core {
  width: 54px;
  height: 54px;
  object-fit: contain;
  cursor: pointer;
  transition: transform 140ms ease-out;
}
.core:hover { transform: translateY(-1.5px) scale(1.03); }
.core:active { transform: scale(0.95); }

.badge {
  position: absolute;
  top: 1px;
  right: 0;
  min-width: 17px;
  height: 17px;
  padding: 0 4px;
  border-radius: 9px;
  background: #9a8b7d;
  color: #fff;
  font-size: 10px;
  line-height: 17px;
  text-align: center;
  font-weight: 600;
  box-shadow: 0 1px 3px rgba(90, 60, 20, 0.3);
}
.badge.warn { background: #e35d3c; }
</style>
