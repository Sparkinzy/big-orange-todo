const pad = (n: number) => String(n).padStart(2, "0")

export function today(): string {
  const d = new Date()
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

export function nowIso(): string {
  const d = new Date()
  return `${today()} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

export function daysLeft(due: string | null): number | null {
  if (!due) return null
  const t = new Date(today() + "T00:00:00").getTime()
  const d = new Date(due + "T00:00:00").getTime()
  return Math.round((d - t) / 86400000)
}

export function dueLabel(due: string | null): string {
  if (!due) return ""
  const n = daysLeft(due)
  if (n === null) return ""
  if (n < 0) return `逾期 ${-n} 天`
  if (n === 0) return "今天"
  if (n === 1) return "明天"
  if (n <= 7) return `${n} 天后`
  return due.slice(5)
}

export function splitDue(input: string): { title: string, due: string | null } {
  const m = input.match(/!(\d{1,2})-(\d{1,2})\s*$/)
  if (!m) return { title: input.trim(), due: null }
  const year = new Date().getFullYear()
  const due = `${year}-${pad(Number(m[1]))}-${pad(Number(m[2]))}`
  return { title: input.replace(/!\d{1,2}-\d{1,2}\s*$/, "").trim(), due }
}
