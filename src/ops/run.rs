use std::path::PathBuf;
use std::process::Stdio;

use tokio::io::AsyncWriteExt;

use crate::context::McContext;
use crate::env::Platform;
use crate::manifest::Manifest;
use crate::utils::errors::McResult;

pub struct RunOptions {
    pub manifest_path: PathBuf
}

pub async fn run(context: &mut McContext, options: &RunOptions) -> McResult<()> {
    let manifest_raw = tokio::fs::read_to_string(&options.manifest_path).await?;

    // TODO: check for eula settings before starting the server that is going to fail
    match toml::from_str::<Manifest>(&manifest_raw) {
        Ok(manifest) => {
            // let java_distribution = manifest.java;
            let current_platform = Platform::current();

            let java_path = match current_platform {
                Platform::Windows => PathBuf::from("java/bin/javaw.exe"),
                Platform::Linux => PathBuf::from("java/bin/java"),
                Platform::MacOS => PathBuf::from("java/Contents/Home/bin/java"),
                Platform::Unknown => {
                    anyhow::bail!("the {} platform is not supported", current_platform)
                }
            };

            let minecraft_path = PathBuf::from("minecraft");

            // TODO: add stdio
            let mut child = tokio::process::Command::new(java_path)
                .arg("-jar")
                .arg("server.jar")
                .arg("--nogui")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .current_dir(&minecraft_path)
                .kill_on_drop(true)
                .spawn()?;

            // Take the pipes
            let mut child_stdin = child.stdin.take().expect("child stdin");
            let child_stdout = child.stdout.take().expect("child stdout");
            let child_stderr = child.stderr.take().expect("child stderr");

            // Pipe child's stdout/stderr -> parent stdout/stderr
            let stdout_task = tokio::spawn(async move {
                let mut out = tokio::io::stdout();

                tokio::io::copy(&mut tokio::io::BufReader::new(child_stdout), &mut out).await
            });

            let stderr_task = tokio::spawn(async move {
                let mut err = tokio::io::stderr();

                tokio::io::copy(&mut tokio::io::BufReader::new(child_stderr), &mut err).await
            });

            // Pipe parent stdin -> child's stdin
            let stdin_task = tokio::spawn(async move {
                let mut input = tokio::io::stdin();

                let _ = tokio::io::copy(&mut input, &mut child_stdin).await;

                // If parent stdin closes, close child's stdin too
                let _ = child_stdin.shutdown().await;

                Ok::<(), std::io::Error>(())
            });

            tokio::select! {
                _ = child.wait() => {

                }
                _ = tokio::signal::ctrl_c() => {
                    child.kill().await?
                    // TODO: ask server to terminate instead of kill.
                    // TODO: fallback to kill, maybe with an extra warning ?
                }
            };

            stdin_task.abort();
            stdout_task.abort();
            stderr_task.abort();

            let _ = tokio::join!(stdin_task, stdout_task, stderr_task);

            // TODO: live backups
            // TODO: make sure version is > alpha v1.0.16_01 before doing live backups
        }
        Err(error) => {
            println!("There is something wrong with the manifest file: {}", error);
        }
    };

    Ok(())
}
