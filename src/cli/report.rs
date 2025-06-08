// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::{load_frames, load_state};
use crate::types::{Frame, Frames, Timespan};
use crate::{Format, ReportArgs};
use chrono::{Datelike, Local, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tabled::{settings::object::Columns, settings::Alignment, settings::Style, Table, Tabled};

#[derive(Debug, Deserialize, Serialize)]
pub struct ReportOutput {
    pub projects: HashMap<String, ProjectDuration>,
    pub total_duration: i64,
    pub timespan: Timespan,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct ProjectDuration {
    pub duration: i64,
}

#[derive(Tabled)]
struct ProjectRow {
    #[tabled(rename = "Project")]
    project: String,
    #[tabled(rename = "Duration")]
    duration: String,
}

impl ReportOutput {
    fn to_text(&self) -> String {
        let from_str = format_timestamp(self.timespan.from);
        let to_str = format_timestamp(self.timespan.to);
        let duration_str = format_duration(self.total_duration);

        if self.projects.is_empty() {
            return format!("From: {from_str}\nTo: {to_str}\n\nTotal: {duration_str}");
        }

        let mut rows: Vec<ProjectRow> = self
            .projects
            .iter()
            .map(|(proj, info)| ProjectRow {
                project: proj.clone(),
                duration: format_duration(info.duration),
            })
            .collect();

        rows.sort_by(|a, b| a.project.cmp(&b.project));

        let mut projects_table = Table::new(rows);
        projects_table
            .with(Style::sharp())
            .modify(Columns::new(1..2), Alignment::right());

        format!("From: {from_str}\nTo: {to_str}\n\n{projects_table}\n\nTotal: {duration_str}")
    }
}

pub fn run_report(args: &ReportArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    if let (Some(from), Some(to)) = (args.from, args.to) {
        if from >= to {
            anyhow::bail!("'to' must be after 'from'");
        }
    }

    let now = Utc::now().timestamp();

    let mut frames = load_frames(config_path)?;
    let state = load_state(config_path)?;

    if let Some(current_frame) = &state.current_frame {
        frames.frames.push(Frame {
            start_time: current_frame.start_time,
            end_time: now,
            project: current_frame.project.clone(),
            updated_at: now,
        });
    }

    let timespan = resolve_timespan(args, now, &frames.frames);

    if timespan.from > timespan.to {
        frames.frames.clear();
    } else {
        filter_frames_by_timespan_from(&mut frames.frames, timespan.from);
        filter_frames_by_timespan_to(&mut frames.frames, timespan.to);
        filter_frames_by_project(&mut frames.frames, &args.project);
    }

    let (project_durations, total_duration) = total_duration_by_project(&frames);

    let output = ReportOutput {
        projects: project_durations,
        total_duration,
        timespan,
    };

    let output_string = match format {
        Format::Json => serde_json::to_string_pretty(&output)?,
        Format::Text => output.to_text(),
    };

    println!("{}", output_string);

    Ok(())
}

pub fn resolve_timespan(args: &ReportArgs, now: i64, frames: &[Frame]) -> Timespan {
    let local_now = Local.timestamp_opt(now, 0).unwrap();

    let from = if args.day {
        local_now.date_naive().and_hms_opt(0, 0, 0).unwrap()
    } else if args.week {
        let weekday = local_now.weekday().num_days_from_monday();
        (local_now.date_naive() - chrono::Duration::days(weekday.into()))
            .and_hms_opt(0, 0, 0)
            .unwrap()
    } else if args.month {
        local_now
            .date_naive()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    } else if args.year {
        local_now
            .date_naive()
            .with_month(1)
            .unwrap()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    } else {
        let from_ts = args
            .from
            .map(|dt| dt.with_timezone(&Utc).timestamp())
            .or_else(|| frames.first().map(|f| f.start_time))
            .unwrap_or(0);
        return Timespan {
            from: from_ts,
            to: args
                .to
                .map(|dt| dt.with_timezone(&Utc).timestamp())
                .unwrap_or(now),
        };
    };

    Timespan {
        from: from
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc)
            .timestamp(),
        to: now,
    }
}

fn filter_frames_by_timespan_from(frames: &mut Vec<Frame>, from: i64) {
    frames.retain_mut(|frame| {
        if frame.start_time < from && frame.end_time > from {
            frame.start_time = from;
            true
        } else {
            frame.start_time >= from
        }
    });
}

fn filter_frames_by_timespan_to(frames: &mut Vec<Frame>, to: i64) {
    frames.retain_mut(|frame| {
        if frame.start_time < to && frame.end_time > to {
            frame.end_time = to;
            true
        } else {
            frame.end_time <= to
        }
    });
}

fn filter_frames_by_project(frames: &mut Vec<Frame>, project: &Option<String>) {
    if let Some(project) = project {
        frames.retain(|frame| frame.project == *project)
    }
}

fn total_duration_by_project(frames: &Frames) -> (HashMap<String, ProjectDuration>, i64) {
    let mut project_durations: HashMap<String, ProjectDuration> = HashMap::new();
    let mut total_time: i64 = 0;

    for frame in &frames.frames {
        let duration = frame.end_time - frame.start_time;
        total_time += duration;

        let entry = project_durations
            .entry(frame.project.clone())
            .or_insert(ProjectDuration { duration: 0 });

        entry.duration += duration;
    }

    (project_durations, total_time)
}

fn format_timestamp(ts: i64) -> String {
    match Local.timestamp_opt(ts, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S (%a)").to_string(),
        chrono::LocalResult::Ambiguous(dt1, _) => dt1.format("%Y-%m-%d %H:%M:%S (%a)").to_string(),
        chrono::LocalResult::None => {
            let fallback_date = NaiveDate::from_ymd_opt(1970, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let fallback_dt = Local.from_local_datetime(&fallback_date).unwrap();
            fallback_dt.format("%Y-%m-%d %H:%M:%S (%a)").to_string()
        }
    }
}

fn format_duration(secs: i64) -> String {
    let mut secs = secs;
    let days = secs / 86400;
    secs %= 86400;
    let hours = secs / 3600;
    secs %= 3600;
    let minutes = secs / 60;
    secs %= 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if secs > 0 || parts.is_empty() {
        parts.push(format!("{}s", secs));
    }

    parts.join(" ")
}
