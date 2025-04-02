use std::{error::Error, process::Command, thread, time::Duration, sync::mpsc::Sender};

type AnyErrorResult<T = ()> = Result<T, Box<dyn Error>>;

pub enum LiveReloadMessage {
  Success,
  BuildFailure,
}

pub fn run_loop(sender: Sender<LiveReloadMessage>) {
  if let Err(e) = run_host(sender) {
    panic!("{e:#}");
  }
}

fn run_host(sender: Sender<LiveReloadMessage>) -> AnyErrorResult {
  let mut build_failed_in_prev_iteration = false;
  loop {
    match build_game_module()? {
      BuildResult::Success => {
        // inserting new line for more clear output of module after compilation failures or previous runs of the module
        println!();
        build_failed_in_prev_iteration = false;

        sender.send(LiveReloadMessage::Success).unwrap();
      }
      BuildResult::Failure => {
        if build_failed_in_prev_iteration {
          continue;
        }
        build_failed_in_prev_iteration = true;
        println!("failed to build the game");

        sender.send(LiveReloadMessage::BuildFailure).unwrap();
      }
      BuildResult::NoChange => {}
    }
    thread::sleep(Duration::from_millis(50));
  }
}

fn build_game_module() -> AnyErrorResult<BuildResult> {
  let output = Command::new("cargo")
    .args(["build", "--package", "game"])
    .output()?;
  let stderr = String::from_utf8(output.stderr)?;

  if !output.status.success() {
    return Ok(BuildResult::Failure);
  }

  Ok(if stderr.contains("Compiling") {
    BuildResult::Success
  } else {
    BuildResult::NoChange
  })
}

enum BuildResult {
  Success,
  Failure,
  NoChange,
}
