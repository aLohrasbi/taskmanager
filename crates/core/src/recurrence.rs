// crates/core/src/recurrence.rs
use crate::{CalendarItem, Result, Task};
use chrono::{DateTime, Duration, Months, Utc};
use std::collections::{HashMap, HashSet};

pub fn occurrence_key(task: &Task, start: &DateTime<Utc>) -> String {
    if task.rrule.is_some() {
        start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    } else {
        "base".into()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Override {
    pub title: Option<String>,
    pub start_at: Option<String>,
    pub cancelled: bool,
}

pub fn parse_overrides(raw: &[(String, String, String)]) -> HashMap<(String, String), Override> {
    let mut out = HashMap::new();
    for (task_id, key, json) in raw {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
            out.insert(
                (task_id.clone(), key.clone()),
                Override {
                    title: v.get("title").and_then(|x| x.as_str()).map(String::from),
                    start_at: v.get("startAt").and_then(|x| x.as_str()).map(String::from),
                    cancelled: v.get("cancelled").and_then(|x| x.as_bool()).unwrap_or(false),
                },
            );
        }
    }
    out
}

// shortcut: supports FREQ=DAILY|WEEKLY|MONTHLY|YEARLY, INTERVAL, COUNT, UNTIL; upgrade to full RFC-5545 crate if BYDAY rules are needed.
fn step(dt: DateTime<Utc>, freq: &str, interval: u32) -> Option<DateTime<Utc>> {
    match freq {
        "DAILY" => Some(dt + Duration::days(interval as i64)),
        "WEEKLY" => Some(dt + Duration::weeks(interval as i64)),
        "MONTHLY" => dt.checked_add_months(Months::new(interval)),
        "YEARLY" => dt.checked_add_months(Months::new(interval * 12)),
        _ => None,
    }
}

fn occurrences(anchor: DateTime<Utc>, rrule: Option<&str>, to: &DateTime<Utc>) -> Vec<DateTime<Utc>> {
    let Some(rule) = rrule.filter(|s| !s.is_empty()) else {
        return vec![anchor];
    };
    let mut freq = "";
    let mut interval = 1u32;
    let mut count = 730usize;
    let mut until: Option<DateTime<Utc>> = None;

    for part in rule.split(';') {
        if let Some((k, v)) = part.split_once('=') {
            match k.trim() {
                "FREQ" => freq = v.trim(),
                "INTERVAL" => interval = v.trim().parse::<u32>().unwrap_or(1).max(1),
                "COUNT" => count = v.trim().parse::<usize>().unwrap_or(730),
                "UNTIL" => until = DateTime::parse_from_rfc3339(v.trim()).ok().map(|d| d.with_timezone(&Utc)),
                _ => {}
            }
        }
    }
    if freq.is_empty() {
        return vec![anchor];
    }

    let mut out = Vec::new();
    let mut cur = anchor;
    for _ in 0..count {
        if cur > *to || until.map_or(false, |u| cur > u) {
            break;
        }
        out.push(cur);
        let Some(next) = step(cur, freq, interval) else { break };
        cur = next;
    }
    out
}

pub fn expand(
    tasks: &[Task],
    from: &DateTime<Utc>,
    to: &DateTime<Utc>,
    completed: &HashSet<(String, String)>,
    overrides: &HashMap<(String, String), Override>,
) -> Result<Vec<CalendarItem>> {
    let mut out = Vec::new();
    for t in tasks {
        let Some(due) = &t.due_at else { continue };
        let anchor = DateTime::parse_from_rfc3339(due)?.with_timezone(&Utc);
        for raw_start in occurrences(anchor, t.rrule.as_deref(), to) {
            let key = occurrence_key(t, &raw_start);
            let ov = overrides.get(&(t.id.clone(), key.clone()));
            if ov.map(|o| o.cancelled).unwrap_or(false) {
                continue;
            }
            let start = ov
                .and_then(|o| o.start_at.as_deref())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or(raw_start);
            if start < *from || start > *to {
                continue;
            }
            let is_done = if t.rrule.is_some() {
                completed.contains(&(t.id.clone(), key.clone()))
            } else {
                t.completed_at.is_some()
            };
            out.push(CalendarItem {
                task_id: t.id.clone(),
                occurrence_key: key,
                title: ov.and_then(|o| o.title.clone()).unwrap_or_else(|| t.title.clone()),
                start_at: start.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                all_day: t.all_day,
                priority: t.priority,
                countdown: t.countdown,
                list_id: t.list_id.clone(),
                completed: is_done,
                recurring: t.rrule.is_some(),
            });
        }
    }
    out.sort_by(|a, b| a.start_at.cmp(&b.start_at));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_daily_rrule_and_tracks_single_occurrence_completion() {
        let t = Task {
            id: "t1".into(),
            list_id: None,
            title: "Vitamin B".into(),
            notes: "".into(),
            due_at: Some("2026-11-14T09:00:00Z".into()),
            due_tz: None,
            all_day: true,
            rrule: Some("FREQ=DAILY;INTERVAL=2;COUNT=3".into()),
            priority: 1,
            countdown: false,
            completed_at: None,
            sort_order: 0,
            created_at: "".into(),
            updated_at: "".into(),
        };
        let from = DateTime::parse_from_rfc3339("2026-11-14T00:00:00Z").unwrap().with_timezone(&Utc);
        let to = DateTime::parse_from_rfc3339("2026-11-20T23:59:59Z").unwrap().with_timezone(&Utc);
        let mut done = HashSet::new();
        done.insert(("t1".into(), "2026-11-14T09:00:00Z".into()));
        let items = expand(&[t], &from, &to, &done, &HashMap::new()).unwrap();
        assert_eq!(items.len(), 3);
        assert!(items[0].completed);
        assert!(!items[1].completed);
        assert!(items[2].start_at.starts_with("2026-11-18"));
    }
}