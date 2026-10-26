<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue"
import { listen } from "@tauri-apps/api/event"
import type { Todo } from "../types"
import { api } from "../api"
import { daysLeft, dueLabel } from "../date"

const props = defineProps<{ id: number }>()

const todo = ref<Todo | null | undefined>(undefined)
const title = ref("")
const content = ref("")
const editingTitle = ref(false)
const titleBox = ref<HTMLInputElement | null>(null)
let saveTimer: number | undefined
let dirty = false

const statusText = computed(() => {
  const t = todo.value
  if (!t) return ""
  if (t.done_at) return "已完成"
  const n = daysLeft(t.due_at)
  if (n === null) return "无日期"
  if (n < 0) return `逾期 ${-n} 天`
  if (n === 0) return "今天到期"
  if (n === 1) return "明天到期"
  return `${n} 天后到期`
})

const dueTone = computed(() => {
  const n = daysLeft(todo.value?.due_at ?? null)
  if (n === null) return ""
  if (n < 0) return "bad"
  if (n === 0) return "soon"
  return ""
})

const createdText = computed(() => todo.value?.created_at ?? "")
const dueText = computed(() => {
  const t = todo.value
  if (!t || !t.due_at) return ""
  return `${dueLabel(t.due_at)}  ·  ${t.due_at.slice(5)}`
})

const editingDue = ref(false)
const dueDraft = ref("")
const dueBox = ref<HTMLInputElement | null>(null)

function startDue() {
  if (!todo.value) return
  dueDraft.value = todo.value.due_at ?? ""
  editingDue.value = true
  nextTick(() => {
    dueBox.value?.focus()
    dueBox.value?.showPicker?.()
  })
}

async function saveDue() {
  if (!todo.value) return
  const v = dueDraft.value || null
  editingDue.value = false
  if (v === todo.value.due_at) return
  await api.update(props.id, todo.value.title, v)
  todo.value.due_at = v
}

async function clearDue() {
  if (!todo.value?.due_at) return
  await api.update(props.id, todo.value.title, null)
  todo.value.due_at = null
}

async function load() {
  const t = await api.get(props.id)
  todo.value = t
  if (t) {
    document.title = t.title
    if (!dirty) {
      title.value = t.title
      content.value = t.content ?? ""
    }
  }
}

async function saveTitle() {
  editingTitle.value = false
  const v = title.value.trim()
  if (!todo.value || !v || v === todo.value.title) {
    title.value = todo.value?.title ?? ""
    return
  }
  await api.update(props.id, v, todo.value.due_at)
  todo.value.title = v
  document.title = v
}

function startEdit() {
  if (!todo.value) return
  editingTitle.value = true
  nextTick(() => {
    titleBox.value?.focus()
    titleBox.value?.select()
  })
}

function onContentInput() {
  dirty = true
  window.clearTimeout(saveTimer)
  saveTimer = window.setTimeout(async () => {
    await api.setContent(props.id, content.value)
    dirty = false
  }, 500)
}

async function toggleDone() {
  if (!todo.value) return
  await api.toggle(props.id, !todo.value.done_at)
}

async function remove() {
  window.clearTimeout(saveTimer)
  await api.remove(props.id)
}

async function close() {
  window.clearTimeout(saveTimer)
  if (todo.value) {
    await api.setContent(props.id, content.value)
    if (todo.value.pinned) await api.setPin(props.id, false)
  }
  await api.closeCard(props.id)
}

watch(() => props.id, load)
onMounted(() => {
  load()
  listen("refresh", load)
})
</script>

<template>
  <div class="card">
    <div v-if="todo === null" class="gone">
      <p>这条待办已经不在了</p>
      <button @click="close">关闭</button>
    </div>

    <template v-else-if="todo">
      <div class="dragbar" data-tauri-drag-region>
        <button class="x" title="关闭" @click="close">
          <svg viewBox="0 0 16 16"><path d="M4.6 4.6l6.8 6.8M11.4 4.6l-6.8 6.8" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"/></svg>
        </button>
      </div>

      <div class="hd">
        <button class="chk" :class="{ on: !!todo.done_at }" @click="toggleDone">
          <svg v-if="todo.done_at" viewBox="0 0 16 16"><path d="M4 8.4 6.6 11 12 5" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/></svg>
        </button>
        <input
          v-if="editingTitle"
          ref="titleBox"
          v-model="title"
          class="title-input"
          @blur="saveTitle"
          @keydown.enter="saveTitle"
          @keydown.esc="editingTitle = false; title = todo!.title"
        />
        <h2 v-else :class="{ done: !!todo.done_at }" @click="startEdit">{{ todo.title }}</h2>
      </div>

      <div class="rows">
        <div class="row">
          <span class="ic">
            <svg viewBox="0 0 20 20">
              <rect x="3" y="4.5" width="14" height="12.5" rx="3" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <path d="M3 8.5h14M6.8 3v3M13.2 3v3" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </span>
          <div class="val">
            <span>{{ createdText }} 创建</span>
            <input
              v-if="editingDue"
              ref="dueBox"
              v-model="dueDraft"
              type="date"
              class="due-input"
              @change="saveDue"
              @blur="editingDue = false"
              @keydown.esc="editingDue = false"
            />
            <span v-else class="chip" :class="dueTone" @click="startDue">
              {{ dueText || "无日期" }}
              <b v-if="todo.due_at" class="clr" title="清除日期" @click.stop="clearDue">×</b>
            </span>
          </div>
        </div>

        <div class="row desc">
          <span class="ic">
            <svg viewBox="0 0 20 20">
              <path d="M5 2.8h6.2L16 7.6v9.6H5z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
              <path d="M11 2.8v5h5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
            </svg>
          </span>
          <textarea
            v-model="content"
            class="content"
            placeholder="补充描述…"
            @input="onContentInput"
          ></textarea>
        </div>
      </div>

      <div class="ft">
        <button class="btn" @click="remove">
          <svg viewBox="0 0 20 20">
            <path d="M4.5 6h11M8 6V4.2h4V6M6.2 6l.7 9.2h6.2L13.8 6" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          删除
        </button>
        <button class="btn primary" @click="toggleDone">
          <svg viewBox="0 0 20 20"><path d="M5 10.6 8.3 14 15 6.6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
          {{ todo.done_at ? "撤销" : "完成" }}
        </button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.card {
  position: relative;
  width: 100%;
  height: 100%;
  background: var(--bg);
  border: 1px solid rgba(200, 160, 120, 0.18);
  border-radius: 22px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 6px 22px rgba(160, 110, 50, 0.16);
  animation: fly-in 260ms cubic-bezier(0.18, 1.1, 0.32, 1) both;
}

.card::after {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: 22px;
  pointer-events: none;
  animation: landing 520ms ease-out both;
}

@keyframes fly-in {
  0% { transform: translate(46px, 32px) scale(0.62); opacity: 0; }
  58% { transform: translate(0, 0) scale(1.022); opacity: 1; }
  100% { transform: translate(0, 0) scale(1); }
}

@keyframes landing {
  0% { box-shadow: inset 0 0 0 2px rgba(245, 166, 35, 0.55); }
  45% { box-shadow: inset 0 0 20px 2px rgba(245, 166, 35, 0.16); }
  100% { box-shadow: inset 0 0 0 0 rgba(245, 166, 35, 0); }
}

.gone {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  color: var(--dim);
  font-size: 12.5px;
}
.gone p { margin: 0; }
.gone button {
  padding: 6px 18px;
  border-radius: 10px;
  background: var(--bg-soft);
  color: var(--text);
  font-size: 12px;
}

.dragbar {
  flex: none;
  height: 28px;
  position: relative;
  -webkit-app-region: drag;
}

.hd {
  flex: none;
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 0 16px;
}

.chk {
  flex: none;
  width: 22px;
  height: 22px;
  margin-top: 1px;
  border: 1.8px solid #e6d6c4;
  border-radius: 7px;
  background: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  -webkit-app-region: no-drag;
  transition: border-color 140ms, background 140ms;
}
.chk:hover { border-color: var(--accent); }
.chk.on {
  background: linear-gradient(160deg, #ffb347, #f5901a);
  border-color: transparent;
}
.chk svg { width: 14px; height: 14px; }

h2 {
  flex: 1;
  min-width: 0;
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  line-height: 24px;
  color: var(--text);
  cursor: text;
  word-break: break-word;
}
h2.done { color: #b3a496; text-decoration: line-through; }

.title-input {
  flex: 1;
  min-width: 0;
  font-size: 15px;
  font-weight: 600;
  line-height: 22px;
  background: #fff;
  border: 1.5px solid var(--accent);
  border-radius: 8px;
  padding: 0 6px;
  -webkit-app-region: no-drag;
}

.x {
  position: absolute;
  right: 8px;
  top: 2px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #c0b0a0;
  -webkit-app-region: no-drag;
  transition: background 140ms, color 140ms;
}
.x svg { width: 14px; height: 14px; }
.x:hover { background: #f6ece0; color: #8d7a6c; }

.rows {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px 16px 10px;
  overflow-y: auto;
  scrollbar-width: thin;
  scrollbar-color: #f0e2d2 transparent;
  scrollbar-gutter: stable;
}
.rows::-webkit-scrollbar { width: 5px; }
.rows::-webkit-scrollbar-track { background: transparent; }
.rows::-webkit-scrollbar-thumb { background: #f0e2d2; border-radius: 3px; }
.rows::-webkit-scrollbar-button { display: none; height: 0; width: 0; }
.rows::-webkit-scrollbar-corner { background: transparent; }

.row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.ic {
  flex: none;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background: #fff3e4;
  color: #b98444;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: -2px;
}
.ic svg { width: 15px; height: 15px; }

.val {
  flex: 1;
  min-width: 0;
  font-size: 12.5px;
  line-height: 22px;
  color: var(--dim);
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}
.chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  margin-top: 1px;
  padding: 1px 7px;
  font-size: 11.5px;
  line-height: 18px;
  color: #b3a496;
  border: 1px dashed #e3d4c3;
  border-radius: 8px;
  cursor: pointer;
  -webkit-app-region: no-drag;
  transition: color 140ms, border-color 140ms, background 140ms;
}
.chip:hover { color: #a9721a; border-color: var(--accent); background: #fff6ea; }
.chip.bad { color: var(--danger); border-color: #f3c2b6; }
.chip.soon { color: var(--warn); border-color: #f7d9a8; }

.clr {
  font-weight: 400;
  font-size: 13px;
  line-height: 1;
  color: #c9b8a8;
  padding: 0 1px;
  border-radius: 4px;
}
.clr:hover { color: var(--danger); background: #ffeeea; }

.due-input {
  width: 138px;
  height: 22px;
  margin-top: 1px;
  padding: 0 5px;
  font-family: inherit;
  font-size: 11.5px;
  color: var(--text);
  background: #fff;
  border: 1.5px solid var(--accent);
  border-radius: 8px;
  outline: none;
  -webkit-app-region: no-drag;
}
.due-input::-webkit-calendar-picker-indicator {
  cursor: pointer;
  opacity: 0.55;
}
.due-input::-webkit-calendar-picker-indicator:hover { opacity: 1; }

.desc {
  flex: 1;
  min-height: 90px;
  align-items: stretch;
}
.content {
  flex: 1;
  min-width: 0;
  min-height: 90px;
  resize: none;
  font-family: inherit;
  font-size: 12.5px;
  line-height: 1.7;
  color: var(--text);
  background: transparent;
  border: none;
  outline: none;
  padding: 0;
  user-select: text;
  -webkit-user-select: text;
}
.content::placeholder { color: #c9b8a8; }

.ft {
  flex: none;
  display: flex;
  gap: 8px;
  padding: 10px 16px 14px;
}

.btn {
  flex: 1;
  height: 36px;
  border-radius: 12px;
  background: #f7f0e7;
  color: #7d6a58;
  font-size: 12.5px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  transition: background 140ms, color 140ms;
}
.btn svg { width: 15px; height: 15px; }
.btn:hover { background: #f2e7d8; color: var(--text); }

.btn.primary {
  background: linear-gradient(160deg, #ffb347, #f5901a);
  color: #fff;
  font-weight: 500;
  box-shadow: 0 2px 8px rgba(240, 140, 20, 0.28);
}
.btn.primary:hover { filter: brightness(1.04); }
</style>
