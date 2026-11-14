// crates/core/src/db.rs
use crate::{new_id, now, Result};
use rusqlite::Connection;
use std::path::Path;

const MIGRATIONS: &[&str] = &[include_str!("../migrations/0001_init.sql")];

pub fn open(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let conn = Connection::open(path)?;
    configure(&conn)?;
    migrate(&conn)?;
    seed_if_empty(&conn)?;
    Ok(conn)
}

pub fn open_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    configure(&conn)?;
    migrate(&conn)?;
    Ok(conn)
}

fn configure(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}

fn migrate(conn: &Connection) -> Result<()> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    for (i, sql) in MIGRATIONS.iter().enumerate() {
        let target = (i + 1) as i64;
        if version < target {
            conn.execute_batch(sql)?;
            conn.pragma_update(None, "user_version", target)?;
        }
    }
    Ok(())
}

fn seed_if_empty(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM lists", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }
    seed(conn)
}

pub fn seed(conn: &Connection) -> Result<()> {
    use chrono::{Duration, Local, NaiveTime, TimeZone, Utc};

    let ts = now();
    let lists = [
        ("Inbox", "#7c83ff"),
        ("Work", "#a855f7"),
        ("Personal", "#7ef0b2"),
        ("Deep Work", "#ffb454"),
    ];
    let mut list_ids = Vec::new();
    for (i, (name, color)) in lists.iter().enumerate() {
        let id = new_id();
        conn.execute(
            "INSERT INTO lists (id,name,color,sort_order,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?5)",
            rusqlite::params![id, name, color, i as i64, ts],
        )?;
        list_ids.push(id);
    }

    let at = |days: i64, h: u32, m: u32| -> String {
        let date = (Local::now() + Duration::days(days)).date_naive();
        let ndt = date.and_time(NaiveTime::from_hms_opt(h, m, 0).unwrap());
        let local = Local.from_local_datetime(&ndt).earliest().unwrap_or_else(Local::now);
        local.with_timezone(&Utc).to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    };

    // title, list idx, day offset, hour, minute, priority, all_day, completed, countdown, rrule
    let tasks: &[(&str, usize, Option<i64>, u32, u32, i64, bool, bool, bool, Option<&str>)] = &[
        ("New Year's Day", 2, Some(48), 0, 0, 3, true, false, true, None),
        ("Reach 95 KG", 2, Some(79), 0, 0, 1, true, false, true, None),
        ("GET 1000$ in bounty", 1, Some(260), 0, 0, 1, true, false, true, None),
        ("25, only gets harder", 2, Some(271), 0, 0, 1, true, false, true, None),
        ("Vitamin B", 2, Some(0), 9, 0, 1, true, false, false, Some("FREQ=DAILY;INTERVAL=2")),
        ("Ship the frosted-widget spike", 1, Some(0), 10, 0, 3, false, false, false, None),
        ("Review pull request #142", 1, Some(0), 14, 30, 2, false, false, false, None),
        ("Inbox zero sweep", 1, Some(0), 17, 0, 1, false, false, false, Some("FREQ=DAILY")),
        ("Call the insurance office", 2, Some(-1), 11, 0, 2, false, false, false, None),
        ("Pay VPS invoice", 2, Some(-2), 9, 0, 3, false, false, false, None),
        ("Water the monstera", 2, Some(0), 8, 0, 0, true, true, false, None),
        ("Draft outline — chapter 2", 3, Some(1), 9, 0, 3, false, false, false, None),
        ("Grocery run: oats, eggs, coffee", 2, Some(1), 18, 0, 0, true, false, false, None),
        ("Read 20 pages", 3, Some(2), 21, 0, 1, false, false, false, None),
        ("Prepare sprint demo notes", 1, Some(2), 11, 0, 2, false, false, false, None),
        ("Book dentist appointment", 2, Some(3), 10, 0, 1, false, false, false, None),
        ("Weekly review + next-week plan", 3, Some(6), 19, 0, 2, false, false, false, Some("FREQ=WEEKLY")),
    ];
    for (i, (title, li, day, h, m, prio, all_day, done, cd, rrule)) in tasks.iter().enumerate() {
        let due = day.map(|d| at(d, *h, *m));
        conn.execute(
            "INSERT INTO tasks (id,list_id,title,due_at,due_tz,all_day,rrule,priority,countdown,completed_at,sort_order,created_at,updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?12)",
            rusqlite::params![
                new_id(), list_ids[*li], title, due, "local", *all_day as i64, *rrule, prio, *cd as i64,
                if *done { Some(at(0, 8, 15)) } else { None }, i as i64, ts
            ],
        )?;
    }

    let habits = [
        ("Read 20 pages", "📚", "#38e1ff", vec![1, 2, 3, 5, 6, 7, 8]),
        ("Morning run", "🏃", "#7ef0b2", vec![1, 2, 3, 4, 5]),
        ("Meditate 10 min", "🧘", "#a78bfa", vec![1, 2, 3, 6, 7]),
        ("No late-night screens", "🌙", "#ffb454", vec![2, 3, 4, 5]),
    ];
    for (name, emoji, color, offsets) in habits {
        let id = new_id();
        conn.execute(
            "INSERT INTO habits (id,name,emoji,color,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?5)",
            rusqlite::params![id, name, emoji, color, ts],
        )?;
        for off in offsets {
            let date = (Local::now() - Duration::days(off)).format("%Y-%m-%d").to_string();
            conn.execute(
                "INSERT INTO habit_entries (habit_id,local_date,value,created_at,updated_at) VALUES (?1,?2,1,?3,?3)",
                rusqlite::params![id, date, ts],
            )?;
        }
    }

    let notes = [
        ("Widget spike — acceptance checklist", 1i64,
         "1. Frameless + transparent\n2. OS blur visible through panel\n3. Stays below normal windows, no focus steal\n4. Survives Win+D / virtual desktops\n5. macOS: desktop level across all Spaces\n6. Two panels: measure RAM + idle CPU"),
        ("Weekly review template", 0, "What shipped?\nWhat stalled and why?\nWhat is the one thing next week?\nClear inbox, reschedule overdue, archive dead tasks."),
    ];
    for (title, pinned, body) in notes {
        conn.execute(
            "INSERT INTO notes (id,title,body,pinned,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?5)",
            rusqlite::params![new_id(), title, body, pinned, ts],
        )?;
    }
    Ok(())
}