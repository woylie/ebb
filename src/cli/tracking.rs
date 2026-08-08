// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::formatting::local_datetime;
use crate::output::{DisplayOutput, print_output};
use crate::persistence::{load_frames, load_state, save_frames, save_state};
use crate::types::{CurrentFrame, Frame, State};
use crate::{Format, RestartArgs, StartArgs, StopArgs};
use anyhow::{Result, bail};
use chrono::{DateTime, Local, TimeZone, Utc};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct StartOutput {
    current_frame: CurrentFrame,
    #[serde(skip_serializing_if = "Option::is_none")]
    stopped_frame: Option<Frame>,
}

impl DisplayOutput for StartOutput {
    fn to_text(&self) -> String {
        let start_datetime = local_datetime(self.current_frame.start_time)
            .expect("timestamps are checked when their file is loaded");

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

impl DisplayOutput for StopOutput {
    fn to_text(&self) -> String {
        let end_datetime = local_datetime(self.stopped_frame.end_time)
            .expect("timestamps are checked when their file is loaded");

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

impl DisplayOutput for CancelOutput {
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

impl DisplayOutput for StatusOutput {
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

    let current_frame = resolve_current_frame(&state, args, now, config_path)?;

    let stopped_frame = if let Some(running_frame) = &state.current_frame {
        let stopped = stop_current_frame(config_path, running_frame, now)?;
        Some(stopped)
    } else {
        None
    };

    state.current_frame = Some(current_frame.clone());
    save_state(config_path, &state)?;

    let output = StartOutput {
        current_frame,
        stopped_frame,
    };

    print_output(&output, format)?;

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

    let current_frame = resolve_current_frame(&state, &start_args, now, config_path)?;
    state.current_frame = Some(current_frame.clone());
    save_state(config_path, &state)?;

    let output = StartOutput {
        current_frame,
        stopped_frame: None,
    };

    print_output(&output, format)?;

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

        print_output(&output, format)?;
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

fn resolve_current_frame(
    state: &State,
    args: &StartArgs,
    now: DateTime<Utc>,
    config_path: &Path,
) -> Result<CurrentFrame> {
    let StartArgs {
        project,
        tags,
        at,
        no_gap,
    } = args;

    // A running frame is stopped at `now`, so `now` precedes the new frame instead of
    // the last end time in frames.toml.
    let (preceding_end, preceding_label) = if state.current_frame.is_some() {
        (Some(now.timestamp()), "the running frame")
    } else if *no_gap || at.is_some() {
        let frames = load_frames(config_path)?;
        (frames.frames.last().map(|f| f.end_time), "the last frame")
    } else {
        (None, "the last frame")
    };

    let start_time = if let Some(at_dt) = at {
        let at_ts = at_dt.with_timezone(&Utc).timestamp();

        if let Some(last_end) = preceding_end
            && at_ts < last_end
        {
            let at_str = at_dt.format("%Y-%m-%d %H:%M:%S").to_string();
            let last_str = chrono::Local
                .timestamp_opt(last_end, 0)
                .single()
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| format!("(invalid timestamp: {})", last_end));

            bail!(
                "Start time ({}) is before the end of {} ({}). \
                    Please specify a later time or omit --at.",
                at_str,
                preceding_label,
                last_str
            );
        }

        at_ts
    } else if *no_gap {
        preceding_end.unwrap_or_else(|| now.timestamp())
    } else {
        now.timestamp()
    };

    Ok(CurrentFrame {
        project: project.to_string(),
        tags: tags.clone(),
        start_time,
    })
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
