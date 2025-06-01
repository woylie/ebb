// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::{load_frames, load_state, save_frames, save_state};
use crate::types::{CurrentFrame, Frame, State};
use crate::StartArgs;
use anyhow::{bail, Result};
use chrono::{DateTime, Local, TimeZone, Utc};
use std::path::Path;

pub fn run_start(args: &StartArgs, config_path: &Path) -> anyhow::Result<()> {
    let mut state = load_state(config_path)?;
    let now = Utc::now();

    if let Some(current_frame) = &state.current_frame {
        stop_current_frame(config_path, current_frame, now)?;
    }

    update_current_frame(&mut state, args, now, config_path)?;
    save_state(config_path, &state)?;

    if let Some(current_frame) = &state.current_frame {
        let start_datetime = Local.timestamp_opt(current_frame.start_time, 0).unwrap();
        let time_str = start_datetime.format("%H:%M:%S").to_string();

        println!(
            "Started project '{}' at {}",
            current_frame.project, time_str
        );
    }

    Ok(())
}

pub fn run_cancel(config_path: &Path) -> anyhow::Result<()> {
    let mut state = load_state(config_path)?;

    if let Some(current_frame) = &state.current_frame.take() {
        save_state(config_path, &state)?;
        println!("Project '{}' cancelled.", current_frame.project);
    } else {
        bail!("No project started.");
    }

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
        at,
        no_gap,
    } = args;

    let mut last_frame_end: Option<i64> = None;

    if *no_gap || at.is_some() {
        if let Ok(frames) = load_frames(config_path) {
            last_frame_end = frames.frames.last().map(|f| f.end_time);
        }
    }

    if at.is_some() && *no_gap {
        bail!("Cannot use --at and --no-gap together.");
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

    let current_frame = CurrentFrame {
        project: project.to_string(),
        start_time,
    };

    state.current_frame = Some(current_frame);
    Ok(())
}

fn stop_current_frame(
    config_path: &Path,
    current_frame: &CurrentFrame,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let start_dt_local = now.with_timezone(&Local);
    let time_str = start_dt_local.format("%H:%M:%S").to_string();
    let mut frames = load_frames(config_path)?;

    let frame = Frame {
        start_time: current_frame.start_time,
        end_time: now.timestamp(),
        project: current_frame.project.clone(),
        updated_at: now.timestamp(),
    };

    frames.frames.push(frame);
    save_frames(config_path, &frames)?;
    println!(
        "Stopped project '{}' at {}",
        current_frame.project, time_str
    );

    Ok(())
}
