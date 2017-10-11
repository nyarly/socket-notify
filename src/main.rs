extern crate regex;
extern crate notify_rust;

#[macro_use]
extern crate error_chain;

use std::os::unix::net::{UnixListener, UnixStream};
use std::fs::{self, File};
use std::thread;
use std::path::Path;
use std::io::Write;
use std::io::{BufRead, BufReader};

use regex::RegexSet;
use notify_rust::Notification;

mod errors {
  error_chain!{
    foreign_links {
      Regex(::regex::Error);
      Io(::std::io::Error);
    }
  }
}

use errors::*;

quick_main!(main_loop);

fn main_loop() -> Result<()> {
  let socket_path = "/tmp/scdaemon.sock";
  let log_path = "/tmp/scdaemon.log";

  // *******
  let socket = Path::new(socket_path);

  // Delete old socket if necessary
  if socket.exists() {
    fs::remove_file(&socket)?;
  }

  // Bind to socket
  let stream = match UnixListener::bind(&socket) {
    Err(_) => panic!("failed to bind socket"),
    Ok(stream) => stream,
  };

  // Iterate over clients, blocks if no client available
  for client in stream.incoming() {
    let log_string = log_path.to_string();
    thread::spawn(|| handle_client(client.unwrap(), log_string));
  }
  return Ok(());
}

fn handle_client(client: UnixStream, log_path: String) -> Result<()> {
  let mut log = File::create(log_path)?;
  let lines = BufReader::new(client).lines();

  let match_set = RegexSet::new(&[r"PK(SIGN|AUTH)", r"result: Success", r"result: "])?;
  let mut notes = Vec::new();

  // | while read line; do
  for line_result in lines {
    let line = line_result?;
    writeln!(log, "{}", line).ok();
    let matches = match_set.matches(&line);
    if matches.matched(0) {
      let note = Notification::new()
        .summary("GPG activity")
        .body("A process is waiting on the Yubikey")
        .appname("scdaemon")
        .timeout(30000)
        .finalize();

      writeln!(log, "Request for signature: notifying.").unwrap_or(());
      match note.show() {
        Ok(handle) => notes.push(handle),
        Err(err) => writeln!(log, "Problem notifying about socket message: {}", err).unwrap_or(()),
      };
      continue;
    }
    if matches.matched(1) {
      writeln!(log, "Recieved success report on socket, notifying").ok();
      match notes.pop() {
        Some(ref mut handle) => {
          handle.body("Accepted!").timeout(200);
          handle.update()
        }
        None => writeln!(log, "No matching notification to update.").unwrap_or(()),
      }
      continue;
    }
    if matches.matched(2) {
      writeln!(log, "Recieved failure report on socket, notifying").ok();
      match notes.pop() {
        Some(ref mut handle) => {
          handle.body("FAILED!").timeout(500);
          handle.update()
        }
        None => writeln!(log, "No matching notification to update.").unwrap_or(()),
      }
      continue;
    }
  }
  return Ok(());
}
