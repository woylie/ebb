// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::{load_frames, load_state, save_frames, save_state};
use crate::types::{CurrentFrame, Frame, State};
use crate::{Format, RestartArgs, StartArgs, StopArgs};
use anyhow::{bail, Result};
use chrono::{DateTime, Local, TimeZone, Utc};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct StartOutput {
    current_frame: CurrentFrame,
    #[serde(skip_serializing_if = "Option::is_none")]
    stopped_frame: Option<Frame>,
}

impl StartOutput {
    fn to_text(&self) -> String {
        let start_datetime = Local
            .timestamp_opt(self.current_frame.start_time, 0)
            .unwrap();

        format!(
            "Project '{}' started at {}.",
            self.current_frame.project,
            start_datetime.format("%H:%M:%S"),
        )
    }
}

#[derive(Serialize)]
struct StopOutput {
    stopped_frame: Frame,
}

impl StopOutput {
    fn to_text(&self) -> String {
        let end_datetime = Local.timestamp_opt(self.stopped_frame.end_time, 0).unwrap();

        format!(
            "Project '{}' stopped at {}.",
            self.stopped_frame.project,
            end_datetime.format("%H:%M:%S"),
        )
    }
}

#[derive(Serialize)]
struct CancelOutput {
    cancelled_frame: CurrentFrame,
}

impl CancelOutput {
    fn to_text(&self) -> String {
        format!(
            "Current frame of project '{}' cancelled.",
            self.cancelled_frame.project
        )
    }
}

#[derive(Serialize)]
struct StatusOutput {
    current_frame: Option<CurrentFrame>,
}

impl StatusOutput {
    fn to_text(&self) -> String {
        if let Some(current_frame) = &self.current_frame {
            let start = match Local.timestamp_opt(current_frame.start_time, 0).single() {
                Some(start) => start,
                None => {
                    return format!(
                        "Current project '{}' has an invalid or ambiguous start time ({}).",
                        current_frame.project, current_frame.start_time
                    );
                }
            };

            let now = Local::now();
            let duration = now.signed_duration_since(start);

            let duration_str = if duration.num_seconds() < 60 {
                "just now".to_string()
            } else if duration.num_hours() == 0 {
                format!("{}m ago", duration.num_minutes())
            } else {
                let hours = duration.num_hours();
                let minutes = duration.num_minutes() % 60;
                format!("{}h {:02}m ago", hours, minutes)
            };

            format!(
                "Current project '{}' started at {} ({}).",
                current_frame.project,
                start.format("%Y-%m-%d %H:%M:%S"),
                duration_str
            )
        } else {
            "No project started.".to_string()
        }
    }
}

pub fn run_start(args: &StartArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let mut state = load_state(config_path)?;
    let now = Utc::now();

    let stopped_frame = if let Some(current_frame) = &state.current_frame {
        let stopped = stop_current_frame(config_path, current_frame, now)?;
        Some(stopped)
    } else {
        None
    };

    update_current_frame(&mut state, args, now, config_path)?;
    save_state(config_path, &state)?;

    if let Some(current_frame) = &state.current_frame {
        let output = StartOutput {
            current_frame: current_frame.clone(),
            stopped_frame,
        };

        let output_string = match format {
            Format::Json => serde_json::to_string_pretty(&output)?,
            Format::Text => output.to_text(),
        };

        println!("{}", output_string);
    }

    Ok(())
}

pub fn run_restart(args: &RestartArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let mut state = load_state(config_path)?;

    if let Some(current_frame) = &state.current_frame {
        bail!(
            "The project '{}' is already in progress.",
            current_frame.project
        );
    }

    let frames = load_frames(config_path)?;

    let Some(last_frame) = frames.frames.last() else {
        bail!("No previous project found to be restarted.");
    };

    let now = Utc::now();

    let start_args = StartArgs {
        at: args.at,
        no_gap: args.no_gap,
        project: last_frame.project.clone(),
        tags: last_frame.tags.clone(),
    };

    update_current_frame(&mut state, &start_args, now, config_path)?;
    save_state(config_path, &state)?;

    if let Some(current_frame) = &state.current_frame {
        let output = StartOutput {
            current_frame: current_frame.clone(),
            stopped_frame: None,
        };

        let output_string = match format {
            Format::Json => serde_json::to_string_pretty(&output)?,
            Format::Text => output.to_text(),
        };

        println!("{}", output_string);
    }

    Ok(())
}

pub fn run_stop(args: &StopArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let mut state = load_state(config_path)?;

    let Some(current_frame) = state.current_frame.take() else {
        bail!("No project started.");
    };

    let StopArgs { at } = args;
    let end_time = if let Some(at) = at {
        if at.timestamp() <= current_frame.start_time {
            let at_str = at.format("%Y-%m-%d %H:%M:%S").to_string();
            let start_time_str = chrono::Local
                .timestamp_opt(current_frame.start_time, 0)
                .single()
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| format!("(invalid timestamp: {})", current_frame.start_time));

            bail!(
                "End time ({}) is before start time ({}). \
        Please specify a later time or omit --at.",
                at_str,
                start_time_str
            );
        }
        at.with_timezone(&Utc)
    } else {
        Utc::now()
    };

    let frame = stop_current_frame(config_path, &current_frame, end_time)?;
    save_state(config_path, &state)?;

    let output = StopOutput {
        stopped_frame: frame,
    };

    let output_string = match format {
        Format::Json => serde_json::to_string_pretty(&output)?,
        Format::Text => output.to_text(),
    };

    println!("{}", output_string);

    Ok(())
}

pub fn run_cancel(config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let mut state = load_state(config_path)?;

    if let Some(current_frame) = &state.current_frame.take() {
        save_state(config_path, &state)?;

        let output = CancelOutput {
            cancelled_frame: current_frame.clone(),
        };

        let output_string = match format {
            Format::Json => serde_json::to_string_pretty(&output)?,
            Format::Text => output.to_text(),
        };

        println!("{}", output_string);
    } else {
        bail!("No project started.");
    }

    Ok(())
}

pub fn run_status(config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let state = load_state(config_path)?;

    let output = StatusOutput {
        current_frame: state.current_frame.clone(),
    };

    let output_string = match format {
        Format::Json => serde_json::to_string_pretty(&output)?,
        Format::Text => output.to_text(),
    };

    println!("{}", output_string);

    Ok(())
}

fn update_current_frame(
    state: &mut State,
    args: &StartArgs,
    now: DateTime<Utc>,
    config_path: &Path,
) -> Result<()> {
    let StartArgs {
        project,
        tags,
        at,
        no_gap,
    } = args;

    let mut last_frame_end: Option<i64> = None;

    if *no_gap || at.is_some() {
        if let Ok(frames) = load_frames(config_path) {
            last_frame_end = frames.frames.last().map(|f| f.end_time);
        }
    }

    let start_time = if let Some(at_dt) = at {
        let at_ts = at_dt.with_timezone(&Utc).timestamp();

        if let Some(last_end) = last_frame_end {
            if at_ts < last_end {
                let at_str = at_dt.format("%Y-%m-%d %H:%M:%S").to_string();
                let last_str = chrono::Local
                    .timestamp_opt(last_end, 0)
                    .single()
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| format!("(invalid timestamp: {})", last_end));

                bail!(
                    "Start time ({}) is before the end of the last frame ({}). \
                    Please specify a later time or omit --at.",
                    at_str,
                    last_str
                );
            }
        }

        at_ts
    } else if *no_gap {
        last_frame_end.unwrap_or_else(|| now.timestamp())
    } else {
        now.timestamp()
    };

    let tags_cleaned: Vec<String> = tags
        .iter()
        .filter_map(|t| t.strip_prefix('+'))
        .map(|s| s.to_string())
        .collect();

    let current_frame = CurrentFrame {
        project: project.to_string(),
        tags: tags_cleaned,
        start_time,
    };

    state.current_frame = Some(current_frame);
    Ok(())
}

fn stop_current_frame(
    config_path: &Path,
    current_frame: &CurrentFrame,
    now: DateTime<Utc>,
) -> anyhow::Result<Frame> {
    let mut frames = load_frames(config_path)?;

    let frame = Frame {
        start_time: current_frame.start_time,
        end_time: now.timestamp(),
        project: current_frame.project.clone(),
        tags: current_frame.tags.clone(),
        updated_at: now.timestamp(),
    };

    frames.frames.push(frame.clone());
    save_frames(config_path, &frames)?;

    Ok(frame)
}
