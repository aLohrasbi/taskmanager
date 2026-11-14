// crates/core/src/habits.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Streak {
    pub current: u32,
    pub best: u32,
    pub done_today: bool,
}

fn day(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

pub fn streak(dates: &[String], today: &str) -> Streak {
    let mut days: Vec<chrono::NaiveDate> = dates.iter().filter_map(|d| day(d)).collect();
    days.sort_unstable();
    days.dedup();
    let Some(t) = day(today) else { return Streak { current: 0, best: 0, done_today: false } };
    if days.is_empty() {
        return Streak { current: 0, best: 0, done_today: false };
    }

    let done_today = days.contains(&t);
    let mut anchor = if done_today { t } else { t.pred_opt().unwrap_or(t) };
    let mut current = 0u32;
    while days.contains(&anchor) {
        current += 1;
        anchor = anchor.pred_opt().unwrap_or(anchor);
    }

    let mut best = 1u32;
    let mut run = 1u32;
    for w in days.windows(2) {
        run = if w[1].signed_duration_since(w[0]).num_days() == 1 { run + 1 } else { 1 };
        best = best.max(run);
    }
    Streak { current, best, done_today }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn d(n: u32) -> String { format!("2026-11-{n:02}") }

    #[test]
    fn empty() {
        assert_eq!(streak(&[], "2026-11-10"), Streak { current: 0, best: 0, done_today: false });
    }
    #[test]
    fn today_only() {
        assert_eq!(streak(&[d(10)], "2026-11-10"), Streak { current: 1, best: 1, done_today: true });
    }
    #[test]
    fn grace_yesterday() {
        let s = streak(&[d(9)], "2026-11-10");
        assert_eq!((s.current, s.done_today), (1, false));
    }
    #[test]
    fn gap_resets_current_keeps_best() {
        let s = streak(&[d(1), d(2), d(3), d(4), d(5), d(10)], "2026-11-10");
        assert_eq!((s.current, s.best), (1, 5));
    }
    #[test]
    fn two_missed_days_kills_it() {
        let s = streak(&[d(1), d(2), d(3)], "2026-11-07");
        assert_eq!((s.current, s.best), (0, 3));
    }
    #[test]
    fn unordered_input() {
        assert_eq!(streak(&[d(10), d(8), d(9)], "2026-11-10").current, 3);
    }
}