// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::formatting::{format_duration, format_timerange};
use crate::output::{DisplayOutput, print_output};
use crate::persistence::{load_frames, load_state};
use crate::types::{Frame, Frames, Timespan};
use crate::{Format, ReportArgs};
use chrono::{Datelike, Local, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tabled::{Table, Tabled, settings::Alignment, settings::Style, settings::object::Columns};

#[derive(Debug, Deserialize, Serialize)]
struct ReportOutput {
    projects: HashMap<String, ProjectDuration>,
    total_duration: i64,
    timespan: Timespan,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct ProjectDuration {
    duration: i64,
    tags: HashMap<String, i64>,
}

#[derive(Tabled)]
struct ProjectRow {
    #[tabled(rename = "Project")]
    project: String,
    #[tabled(rename = "Duration")]
    duration: String,
}

impl DisplayOutput for ReportOutput {
    fn to_text(&self) -> String {
        let timerange_str = format_timerange(self.timespan.from, self.timespan.to);
        let duration_str = format_duration(self.total_duration);

        if self.projects.is_empty() {
            return format!("{timerange_str}\n\nTotal: {duration_str}");
        }

        let mut rows: Vec<ProjectRow> = Vec::new();
        let mut project_names: Vec<_> = self.projects.keys().collect();
        project_names.sort();

        for project in project_names {
            let info = &self.projects[project];
            rows.push(ProjectRow {
                project: project.clone(),
                duration: format_duration(info.duration),
            });

            let mut tags: Vec<_> = info.tags.iter().collect();
            tags.sort_by_key(|(tag, _)| *tag);

            for (tag, &duration) in tags {
                rows.push(ProjectRow {
                    project: format!("  +{}", tag),
                    duration: format_duration(duration),
                });
            }
        }

        let mut table = Table::new(rows);
        table
            .with(Style::sharp())
            .modify(Columns::new(1..2), Alignment::right());

        format!("{timerange_str}\n\n{table}\n\nTotal: {duration_str}")
    }
}

pub fn run_report(args: &ReportArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    if let (Some(from), Some(to)) = (args.from, args.to)
        && from >= to
    {
        anyhow::bail!("'to' must be after 'from'");
    }

    let now = Utc::now().timestamp();

    let mut frames = load_frames(config_path)?;
    let state = load_state(config_path)?;

    if let Some(current_frame) = &state.current_frame {
        frames.frames.push(Frame {
            start_time: current_frame.start_time,
            end_time: now,
            project: current_frame.project.clone(),
            tags: current_frame.tags.clone(),
            updated_at: now,
        });
    }

    let timespan = resolve_timespan(args, now, &frames.frames);

    if timespan.from > timespan.to {
        frames.frames.clear();
    } else {
        frames
            .filter_by_start_time(timespan.from)
            .filter_by_end_time(timespan.to);

        if let Some(ref project) = args.project {
            frames.filter_by_project(project);
        }
        if let Some(ref tag) = args.tag {
            frames.filter_by_tag(tag);
        }
    }

    let (project_durations, total_duration) = total_duration_by_project(&frames);

    let output = ReportOutput {
        projects: project_durations,
        total_duration,
        timespan,
    };

    print_output(&output, format)?;

    Ok(())
}

fn resolve_timespan(args: &ReportArgs, now: i64, frames: &[Frame]) -> Timespan {
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

fn total_duration_by_project(frames: &Frames) -> (HashMap<String, ProjectDuration>, i64) {
    let mut project_durations: HashMap<String, ProjectDuration> = HashMap::new();
    let mut total_time: i64 = 0;

    for frame in &frames.frames {
        let duration = frame.end_time - frame.start_time;
        total_time += duration;

        let entry = project_durations
            .entry(frame.project.clone())
            .or_insert(ProjectDuration {
                duration: 0,
                tags: HashMap::new(),
            });

        entry.duration += duration;

        for tag in &frame.tags {
            *entry.tags.entry(tag.clone()).or_insert(0) += duration;
        }
    }

    (project_durations, total_time)
}
