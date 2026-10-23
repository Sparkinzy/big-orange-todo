use chrono::Local;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Serialize, Clone)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub content: Option<String>,
    pub category: Option<String>,
    pub created_at: String,
    pub due_at: Option<String>,
    pub done_at: Option<String>,
    pub pinned: bool,
}

pub struct Db(pub Mutex<Connection>);

fn data_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let d = dir.join("data");
            if std::fs::create_dir_all(&d).is_ok() {
                let probe = d.join(".probe");
                if std::fs::write(&probe, b"1").is_ok() {
                    let _ = std::fs::remove_file(probe);
                    return d;
                }
            }
        }
    }
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    let d = PathBuf::from(base).join("StickTodo");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn open() -> Connection {
    let c = Connection::open(data_dir().join("todo.db")).expect("打开数据库失败");
    c.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         CREATE TABLE IF NOT EXISTS todos (
           id INTEGER PRIMARY KEY AUTOINCREMENT,
           title TEXT NOT NULL,
           content TEXT,
           created_at TEXT NOT NULL,
           due_at TEXT,
           done_at TEXT,
           pinned INTEGER NOT NULL DEFAULT 0,
           px INTEGER,
           py INTEGER
         );
         CREATE INDEX IF NOT EXISTS idx_open ON todos(done_at, due_at);",
    )
    .expect("建表失败");
    // 老库补列
    let _ = c.execute("ALTER TABLE todos ADD COLUMN content TEXT", []);
    let _ = c.execute("ALTER TABLE todos ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0", []);
    let _ = c.execute("ALTER TABLE todos ADD COLUMN px INTEGER", []);
    let _ = c.execute("ALTER TABLE todos ADD COLUMN category TEXT", []);
    let _ = c.execute("ALTER TABLE todos ADD COLUMN py INTEGER", []);
    c
}

pub fn now_text() -> String {
    Local::now().format("%Y-%m-%d %H:%M").to_string()
}

const COLS: &str = "id, title, content, category, created_at, due_at, done_at, pinned";

fn row_to_todo(r: &rusqlite::Row) -> rusqlite::Result<Todo> {
    Ok(Todo {
        id: r.get(0)?,
        title: r.get(1)?,
        content: r.get(2)?,
        category: r.get(3)?,
        created_at: r.get(4)?,
        due_at: r.get(5)?,
        done_at: r.get(6)?,
        pinned: r.get::<_, i64>(7)? != 0,
    })
}

pub fn all(c: &Connection) -> Vec<Todo> {
    let mut st = match c.prepare(&format!("SELECT {COLS} FROM todos")) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let rows: Vec<Todo> = match st.query_map([], row_to_todo) {
        Ok(it) => it.filter_map(Result::ok).collect(),
        Err(_) => vec![],
    };
    rows
}

pub fn get(c: &Connection, id: i64) -> Option<Todo> {
    c.query_row(&format!("SELECT {COLS} FROM todos WHERE id = ?1"), params![id], row_to_todo)
        .ok()
}

pub fn add(c: &Connection, title: &str, due: Option<&str>) -> Option<Todo> {
    let now = now_text();
    c.execute(
        "INSERT INTO todos (title, created_at, due_at) VALUES (?1, ?2, ?3)",
        params![title, now, due],
    )
    .ok()?;
    get(c, c.last_insert_rowid())
}

pub fn toggle(c: &Connection, id: i64, done: bool) {
    let at = if done { Some(now_text()) } else { None };
    let _ = c.execute("UPDATE todos SET done_at = ?1 WHERE id = ?2", params![at, id]);
}

pub fn update(c: &Connection, id: i64, title: &str, due: Option<&str>) {
    let _ = c.execute(
        "UPDATE todos SET title = ?1, due_at = ?2 WHERE id = ?3",
        params![title, due, id],
    );
}

pub fn set_category(c: &Connection, id: i64, category: Option<&str>) {
    let _ = c.execute(
        "UPDATE todos SET category = ?1 WHERE id = ?2",
        params![category, id],
    );
}

pub fn set_content(c: &Connection, id: i64, content: &str) {
    let _ = c.execute(
        "UPDATE todos SET content = ?1 WHERE id = ?2",
        params![content, id],
    );
}

pub fn set_pin(c: &Connection, id: i64, pinned: bool, x: i32, y: i32) {
    let _ = c.execute(
        "UPDATE todos SET pinned = ?1, px = ?2, py = ?3 WHERE id = ?4",
        params![if pinned { 1 } else { 0 }, x, y, id],
    );
}

pub fn pinned_ids(c: &Connection) -> Vec<(i64, Option<i32>, Option<i32>)> {
    let mut st = match c.prepare("SELECT id, px, py FROM todos WHERE pinned = 1") {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let rows: Vec<(i64, Option<i32>, Option<i32>)> =
        match st.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))) {
            Ok(it) => it.filter_map(Result::ok).collect(),
            Err(_) => vec![],
        };
    rows
}

pub fn remove(c: &Connection, id: i64) {
    let _ = c.execute("DELETE FROM todos WHERE id = ?1", params![id]);
}
