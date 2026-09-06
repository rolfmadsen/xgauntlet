//! Multi-layer verification runner executing commands with timeouts and exit codes.

use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command;

use super::models::{
    GauntletReport, LayerDefinition, LayerExecutionStatus, LayerRequirement, LayerResult,
};

/// Executes a single verification layer and captures output, exit code, and execution status.
pub async fn execute_layer(layer: &LayerDefinition, cwd: &Path) -> LayerResult {
    let start_time = Instant::now();
    if layer.command.is_empty() {
        return LayerResult {
            name: layer.name.clone(),
            exit_code: 1,
            passed: false,
            output: "Layer definition contains an empty command".to_string(),
            status: LayerExecutionStatus::Error,
            duration_seconds: 0.0,
            requirement: layer.requirement,
        };
    }

    let mut cmd = Command::new(&layer.command[0]);
    if layer.command.len() > 1 {
        cmd.args(&layer.command[1..]);
    }
    cmd.current_dir(cwd);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let duration = start_time.elapsed().as_secs_f64();
            let exit_code = if e.kind() == std::io::ErrorKind::NotFound {
                127
            } else {
                1
            };
            return LayerResult {
                name: layer.name.clone(),
                exit_code,
                passed: false,
                output: format!("Layer execution failed (command spawn error): {e}"),
                status: if exit_code == 127 {
                    LayerExecutionStatus::Unavailable
                } else {
                    LayerExecutionStatus::Error
                },
                duration_seconds: duration,
                requirement: layer.requirement,
            };
        }
    };

    let stdout_handle = child.stdout.take();
    let stderr_handle = child.stderr.take();
    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();

    let timeout_duration = Duration::from_secs_f64(layer.timeout_seconds.max(0.001));

    tokio::select! {
        res = async {
            let mut out_buf = Vec::new();
            let mut err_buf = Vec::new();

            let out_task = async {
                if let Some(mut out) = stdout_handle {
                    let _ = out.read_to_end(&mut out_buf).await;
                }
                out_buf
            };

            let err_task = async {
                if let Some(mut err) = stderr_handle {
                    let _ = err.read_to_end(&mut err_buf).await;
                }
                err_buf
            };

            let (o_buf, e_buf) = tokio::join!(out_task, err_task);
            stdout_buf = o_buf;
            stderr_buf = e_buf;

            child.wait().await
        } => {
            let duration = start_time.elapsed().as_secs_f64();
            match res {
                Ok(status) => {
                    let exit_code = status.code().unwrap_or(if status.success() { 0 } else { 1 });
                    let passed = status.success();
                    let stdout_str = String::from_utf8_lossy(&stdout_buf);
                    let stderr_str = String::from_utf8_lossy(&stderr_buf);
                    let combined = format!("{stdout_str}{stderr_str}");
                    LayerResult {
                        name: layer.name.clone(),
                        exit_code,
                        passed,
                        output: combined,
                        status: if passed {
                            LayerExecutionStatus::Passed
                        } else {
                            LayerExecutionStatus::Failed
                        },
                        duration_seconds: duration,
                        requirement: layer.requirement,
                    }
                }
                Err(e) => {
                    LayerResult {
                        name: layer.name.clone(),
                        exit_code: 1,
                        passed: false,
                        output: format!("Layer process wait error: {e}"),
                        status: LayerExecutionStatus::Error,
                        duration_seconds: duration,
                        requirement: layer.requirement,
                    }
                }
            }
        }
        _ = tokio::time::sleep(timeout_duration) => {
            let _ = child.start_kill();
            let duration = start_time.elapsed().as_secs_f64();
            LayerResult {
                name: layer.name.clone(),
                exit_code: 124,
                passed: false,
                output: format!(
                    "Layer '{}' timed out after {:.2}s",
                    layer.name, layer.timeout_seconds
                ),
                status: LayerExecutionStatus::TimedOut,
                duration_seconds: duration,
                requirement: layer.requirement,
            }
        }
    }
}

/// Executes verification layers sequentially with fail-closed semantics on required failures.
pub async fn run_gauntlet(layers: &[LayerDefinition], cwd: &Path) -> GauntletReport {
    let overall_start = Instant::now();
    let mut results = Vec::new();
    let mut success = true;

    for layer in layers {
        let res = execute_layer(layer, cwd).await;
        results.push(res.clone());

        if !res.passed && layer.requirement == LayerRequirement::Required && !layer.optional {
            success = false;
            break;
        }
    }

    let total_duration = overall_start.elapsed().as_secs_f64();
    GauntletReport::new(success, results, total_duration)
}
