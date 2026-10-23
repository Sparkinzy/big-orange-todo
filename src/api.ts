import { invoke } from "@tauri-apps/api/core"
import type { Todo } from "./types"

export const api = {
  list: () => invoke<Todo[]>("list_todos"),
  get: (id: number) => invoke<Todo | null>("get_todo", { id }),
  add: (title: string, due: string | null) => invoke<Todo>("add_todo", { title, due }),
  toggle: (id: number, done: boolean) => invoke<void>("toggle_todo", { id, done }),
  update: (id: number, title: string, due: string | null) => invoke<void>("update_todo", { id, title, due }),
  setContent: (id: number, content: string) => invoke<void>("set_content", { id, content }),
  setCategory: (id: number, category: string | null) => invoke<void>("set_category", { id, category }),
  remove: (id: number) => invoke<void>("delete_todo", { id }),
  openCard: (id: number) => invoke<void>("open_card", { id }),
  closeCard: (id: number) => invoke<void>("close_card", { id }),
  setPin: (id: number, pinned: boolean) => invoke<void>("set_pin", { id, pinned }),
  setMode: (mode: string) => invoke<void>("set_mode", { mode }),
  expand: () => invoke<void>("expand"),
  collapse: () => invoke<void>("collapse"),
  hide: () => invoke<void>("hide_window"),
  quit: () => invoke<void>("quit_app")
}
