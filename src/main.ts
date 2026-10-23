import { createApp } from "vue"
import { getCurrentWindow } from "@tauri-apps/api/window"
import App from "./App.vue"
import Card from "./components/Card.vue"
import "./styles.css"

let cardId: number | null = null

try {
  const label = getCurrentWindow().label
  if (label.startsWith("card-")) {
    const n = Number(label.slice(5))
    if (Number.isFinite(n)) cardId = n
  }
} catch {
  cardId = null
}

if (cardId !== null) {
  createApp(Card, { id: cardId }).mount("#app")
} else {
  createApp(App).mount("#app")
}
