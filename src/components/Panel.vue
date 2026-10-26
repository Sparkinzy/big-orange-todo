<script setup lang="ts">
import { computed, onMounted, ref } from "vue"
import { listen } from "@tauri-apps/api/event"
import type { Todo } from "../types"
import { api } from "../api"
import { daysLeft, dueLabel, splitDue } from "../date"
import peekCatL from "../assets/cat/peek-cat.png"
import peekCatR from "../assets/cat/peek-cat-flip.png"
import boxOff from "../assets/cat/box-off.png"
import boxOn from "../assets/cat/box-on.png"
import trashIcon from "../assets/cat/btn-trash.png"
import closeIcon from "../assets/cat/btn-x.png"
import sendIcon from "../assets/cat/btn-send.png"
import ballIcon from "../assets/cat/badge-cat-2.png"
import catBody from "../assets/cat/cat-body.png"

const props = defineProps<{
  todos: Todo[]
  side: string
  collapsed: boolean
}>()

const emit = defineEmits<{
  (e: "expand"): void
  (e: "collapse"): void
  (e: "add", title: string, due: string | null): void
  (e: "toggle", t: Todo): void
  (e: "remove", id: number): void
  (e: "open-card", id: number): void
  (e: "to-ball"): void
}>()

const draft = ref("")
const tab = ref<"open" | "done">("open")
const hoverId = ref<number | null>(null)
/** 正在打开的卡片：卡片渲染完会回一个 card-ready，先把行切成「打开中」 */
const openingId = ref<number | null>(null)

onMounted(() => listen<number>("card-ready", () => (openingId.value = null)))

function openCard(id: number) {
  openingId.value = id
  emit("open-card", id)
}

const openTodos = computed(() => props.todos.filter(t => !t.done_at))
const doneCount = computed(() => props.todos.filter(t => !!t.done_at).length)

function bucket(t: Todo): number {
  if (t.done_at) return 9
  const n = daysLeft(t.due_at)
  if (n === null) return 3
  if (n < 0) return 0
  if (n === 0) return 1
  return 2
}

const rows = computed(() => {
  const list = props.todos.filter(t => (tab.value === "open" ? !t.done_at : !!t.done_at))
  return list.sort((a, b) => {
    const ba = bucket(a)
    const bb = bucket(b)
    if (ba !== bb) return ba - bb
    if (ba === 2) return a.due_at! < b.due_at! ? -1 : 1
    if (ba === 9) return a.done_at! < b.done_at! ? 1 : -1
    return a.created_at < b.created_at ? 1 : -1
  })
})

function submit() {
  const raw = draft.value.trim()
  if (!raw) return
  const { title, due } = splitDue(raw)
  if (!title) return
  emit("add", title, due)
  draft.value = ""
}

function dueClass(t: Todo) {
  const n = daysLeft(t.due_at)
  if (n === null) return ""
  if (n < 0) return "bad"
  if (n === 0) return "soon"
  return ""
}

function metaText(t: Todo) {
  return t.done_at ? `完成于 ${t.done_at.slice(5)}` : `${t.created_at.slice(5)} 创建`
}
</script>

<template>
  <div class="wrap">
    <div v-if="collapsed" class="peek" :class="side" @click="emit('expand')">
      <img class="cling" :src="side === 'left' ? peekCatL : peekCatR" alt="" draggable="false" />
    </div>

    <div v-else class="panel" :style="{ '--from': side === 'left' ? '-100%' : '100%' }">
      <div class="topbar" data-tauri-drag-region>
        <span class="tico">
          <svg viewBox="0 0 16 16"><path d="M3.5 8.4 6.4 11.2 12.5 4.8" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
        </span>
        <span class="ttitle">{{ tab === "open" ? "今日待办" : "已完成" }}</span>
        <span class="tcount">{{ doneCount }}/{{ todos.length }}</span>
        <div class="acts">
          <button class="ico" title="切换悬浮球" @click="emit('to-ball')">
            <img :src="ballIcon" alt="" />
          </button>
          <button class="ico x" title="收起来（退出程序请用托盘菜单）" @click="emit('collapse')">
            <img :src="closeIcon" alt="" />
          </button>
        </div>
      </div>

      <div class="sheet">
        <ul class="list">
          <li
            v-for="t in rows"
            :key="t.id"
            :class="{ done: !!t.done_at, hot: hoverId === t.id, od: !t.done_at && (daysLeft(t.due_at) ?? 1) < 0 }"
            @mouseenter="hoverId = t.id"
            @mouseleave="hoverId = null"
          >
            <button class="chk" :class="{ on: !!t.done_at }" @click="emit('toggle', t)">
              <img :src="t.done_at ? boxOn : boxOff" alt="" />
            </button>

            <div class="body" @dblclick="openCard(t.id)">
              <div class="line1">
                <span class="title">{{ t.title }}</span>
                <span v-if="t.due_at && !t.done_at" class="due" :class="dueClass(t)">{{ dueLabel(t.due_at) }}</span>
              </div>
              <div class="meta">{{ metaText(t) }}</div>
            </div>

            <span v-if="openingId === t.id" class="spin" title="打开中"></span>
            <button v-else-if="hoverId === t.id" class="del" title="删除" @click="emit('remove', t.id)">
              <img :src="trashIcon" alt="" />
            </button>
          </li>
        </ul>

        <div v-if="!rows.length" class="empty">
          <img :src="catBody" alt="" />
          <p>{{ tab === "open" ? "还没有待办" : "还没有完成的" }}</p>
        </div>

        <div class="composer">
          <svg class="plus" viewBox="0 0 16 16"><path d="M8 3.5v9M3.5 8h9" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/></svg>
          <input
            v-model="draft"
            placeholder="添加待办..."
            @keydown.enter="submit"
            @keydown.esc="draft = ''"
          />
          <button class="send" title="添加" @click="submit">
            <img :src="sendIcon" alt="" />
          </button>
        </div>

        <div class="tabs">
          <button :class="{ on: tab === 'open' }" @click="tab = 'open'">未完成 {{ openTodos.length }}</button>
          <button :class="{ on: tab === 'done' }" @click="tab = 'done'">已完成 {{ doneCount }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.wrap {
  width: 100%;
  height: 100%;
}

.peek {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  overflow: hidden;
  cursor: pointer;
}
.peek.left { justify-content: flex-start; }

/* 趴在窗口边缘的小猫：素材里那条白边就是「窗框」，贴屏幕边那一侧，猫身朝屏幕里。
   左右各出一张镜像图，避免靠 transform 翻面把动画带歪。 */
/* 定死像素尺寸：收起动画里窗口还是 340x520 时，图不会被拉伸成巨猫 */
.cling {
  flex: none;
  width: 96px;
  height: 117px;
  transform-origin: 88% 58%;
  filter: drop-shadow(-3px 4px 7px rgba(150, 90, 20, 0.3));
  animation: cling 3.8s ease-in-out infinite;
  transition: scale 160ms ease-out;
}
.peek.left .cling {
  transform-origin: 12% 58%;
  filter: drop-shadow(3px 4px 7px rgba(150, 90, 20, 0.3));
}

.peek:hover .cling { scale: 1.04; }
.peek:active .cling { scale: 0.97; }

@keyframes cling {
  0%, 100% { translate: 0 0; rotate: 0deg; }
  38% { translate: -1px 3px; rotate: 1.8deg; }
  70% { translate: 0 1px; rotate: 0.6deg; }
}


/* ---------- 面板 ---------- */

.panel {
  width: 100%;
  height: 100%;
  background: linear-gradient(168deg, #f7ad3f 0%, #f29c22 100%);
  border-radius: 22px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  will-change: transform;
  animation: slide-in 230ms cubic-bezier(0.22, 0.86, 0.24, 1) both;
}

/* 从吸附的那条屏幕边滑进来，而不是原地弹一下 */
@keyframes slide-in {
  from { transform: translateX(var(--from, 100%)); }
  to { transform: translateX(0); }
}

.topbar {
  flex: none;
  height: 54px;
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 0 10px 0 14px;
  -webkit-app-region: drag;
}

.tico {
  flex: none;
  width: 24px;
  height: 24px;
  border-radius: 7px;
  background: #fff;
  color: #f5a623;
  display: flex;
  align-items: center;
  justify-content: center;
}
.tico svg { width: 14px; height: 14px; }

.ttitle {
  font-size: 15px;
  font-weight: 600;
  color: #fff;
  letter-spacing: 0.3px;
  text-shadow: 0 1px 2px rgba(150, 90, 0, 0.18);
}

.tcount {
  font-size: 12.5px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.82);
}

.acts {
  margin-left: auto;
  display: flex;
  gap: 2px;
  -webkit-app-region: no-drag;
}
.acts button {
  width: 31px;
  height: 31px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 120ms, filter 140ms;
}
.acts button img { width: 100%; height: 100%; display: block; }
.acts button:hover { filter: brightness(1.05); transform: translateY(-1px); }
.acts button:active { transform: scale(0.94); }

.sheet {
  flex: 1;
  min-height: 0;
  margin: 0 9px 9px;
  background: #fff;
  border-radius: 17px;
  display: flex;
  flex-direction: column;
  padding: 6px 12px 8px;
  box-shadow: 0 2px 10px rgba(160, 110, 50, 0.1);
}

.list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  margin: 0;
  padding: 0;
  list-style: none;
  scrollbar-width: thin;
}
.list::-webkit-scrollbar { width: 5px; }
.list::-webkit-scrollbar-thumb { background: #f0e2d2; border-radius: 3px; }

.list li {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 11px;
  padding: 11px 6px 11px 4px;
  border-radius: 12px;
  transition: background 130ms;
}
.list li.hot { background: #fff8ee; }
.list li.od::before {
  content: "";
  position: absolute;
  left: -6px;
  top: 14px;
  bottom: 14px;
  width: 3px;
  border-radius: 2px;
  background: #f0836a;
}

.chk {
  flex: none;
  width: 22px;
  height: 22px;
  margin-top: 1px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 130ms;
}
.chk img { width: 100%; height: 100%; display: block; }
.chk:hover { transform: scale(1.08); }
.chk:active { transform: scale(0.92); }

.body { flex: 1; min-width: 0; cursor: text; }

.line1 {
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.title {
  flex: 1;
  min-width: 0;
  font-size: 13.5px;
  font-weight: 500;
  line-height: 19px;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.due {
  flex: none;
  font-size: 11px;
  color: #a99a8c;
  font-feature-settings: "tnum";
}
.due.bad { color: var(--danger); }
.due.soon { color: var(--warn); }

.meta {
  margin-top: 3px;
  font-size: 11px;
  line-height: 15px;
  color: #b3a496;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.done .title { color: #b3a496; text-decoration: line-through; }
.done .due { display: none; }

.del {
  flex: none;
  width: 23px;
  height: 23px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 130ms;
}
.del img { width: 100%; height: 100%; display: block; }
.del:hover { transform: scale(1.1); }

/* 打开卡片时行尾的转圈：卡片窗口要几百毫秒才出内容，先让点击有回应 */
.spin {
  flex: none;
  width: 15px;
  height: 15px;
  margin-top: 4px;
  border: 2px solid #f6e2c6;
  border-top-color: #f5a623;
  border-radius: 50%;
  animation: spin 620ms linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
.list li:has(.spin) .title { color: #b98444; }

.empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 60px;
  color: #d6c7b8;
}
.empty img { width: 84px; height: auto; }
.empty p { margin: 10px 0 0; font-size: 12.5px; }

.composer {
  flex: none;
  display: flex;
  align-items: center;
  gap: 9px;
  height: 44px;
  margin: 8px 2px 6px;
  padding: 0 4px 0 12px;
  border: 1.6px solid #f2e5d6;
  border-radius: 14px;
  background: #fffdfa;
  transition: border-color 150ms;
}
.composer:focus-within { border-color: var(--accent); }
.composer .plus { width: 17px; height: 17px; flex: none; color: #b98444; }
.composer input { flex: 1; height: 100%; font-size: 12.5px; }

.send {
  flex: none;
  width: 34px;
  height: 34px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 120ms, filter 140ms;
}
.send img { width: 100%; height: 100%; display: block; }
.send:hover { filter: brightness(1.05); }
.send:active { transform: scale(0.92); }

.tabs {
  flex: none;
  display: flex;
  gap: 4px;
  padding: 0 2px 2px;
}
.tabs button {
  font-size: 11px;
  padding: 4px 9px;
  border-radius: 8px;
  color: #b3a496;
  transition: background 130ms, color 130ms;
}
.tabs button:hover { color: var(--text); }
.tabs button.on {
  background: #fff4e6;
  color: #a9721a;
  font-weight: 500;
}
</style>
