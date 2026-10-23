export interface Todo {
  id: number
  title: string
  content: string | null
  category: string | null
  created_at: string
  due_at: string | null
  done_at: string | null
  pinned: boolean
}
