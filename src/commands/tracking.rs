// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::types::{CurrentFrame, Frame, Frames, State};
use crate::StartArgs;
use anyhow::{bail, Result};
use chrono::{DateTime, Local, TimeZone, Utc};
use std::fs;
use std::path::Path;

const FRAMES_FILE: &str = "frames.toml";
const STATE_FILE: &str = "state.toml";

pub fn run_start(args: &StartArgs, config_path: &Path) -> anyhow::Result<()> {
    let state_file = config_path.join(STATE_FILE);
    let mut state = load_state(&state_file)?;
    let now = Utc::now();

    if let Some(current_frame) = &state.current_frame {
        stop_current_frame(config_path, current_frame, now)?;
    }

    update_current_frame(&mut state, args, now, config_path)?;
    save_state(&state_file, &state)?;

    if let Some(current_frame) = &state.current_frame {
        let start_datetime = Local.timestamp_opt(current_frame.start_time, 0).unwrap();
        let time_str = start_datetime.format("%H:%M:%S").to_string();

        println!("Started project {} at {}", current_frame.project, time_str);
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
        let frames_file = config_path.join(FRAMES_FILE);
        if let Ok(frames) = load_frames(&frames_file) {
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

    let frames_file = config_path.join(FRAMES_FILE);
    let mut frames = load_frames(&frames_file)?;

    let frame = Frame {
        start_time: current_frame.start_time,
        end_time: now.timestamp(),
        project: current_frame.project.clone(),
        updated_at: now.timestamp(),
    };

    frames.frames.push(frame);
    save_frames(&frames_file, &frames)?;
    println!("Stopped project {} at {}", current_frame.project, time_str);

    Ok(())
}

fn load_frames(path: &Path) -> Result<Frames> {
    if !path.exists() {
        return Ok(Frames { frames: Vec::new() });
    }
    let contents = fs::read_to_string(path)?;
    Ok(toml::from_str(&contents)?)
}

fn save_frames(path: &Path, frames: &Frames) -> Result<()> {
    let toml = toml::to_string(&frames)?;
    fs::write(path, toml)?;
    Ok(())
}

fn load_state(path: &Path) -> Result<State> {
    if !path.exists() {
        return Ok(State {
            current_frame: None,
        });
    }
    let contents = fs::read_to_string(path)?;
    Ok(toml::from_str(&contents)?)
}

fn save_state(path: &Path, state: &State) -> Result<()> {
    let toml = toml::to_string(&state)?;
    fs::write(path, toml)?;
    Ok(())
}
