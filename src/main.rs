extern crate regex;
extern crate notify_rust;

use std::os::unix::net::UnixStream;
use std::{fmt, error};
use std::process::exit;
use std::fs::File;
use std::io::Write;
use std::io::{self, BufRead, BufReader};

use regex::RegexSet;
use notify_rust::Notification;

#[derive(Debug)]
enum Error {
  Io(io::Error),
  Regex(regex::Error),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Error::Io(ref err) => write!(f, "{:?}", err),
      Error::Regex(ref err) => write!(f, "{:?}", err),
    }
  }
}

impl error::Error for Error {
  fn description(&self) -> &str {
    match *self {
      Error::Io(ref err) => err.description(),
      Error::Regex(ref err) => err.description(),
    }
  }

  fn cause(&self) -> Option<&error::Error> {
    match *self {
      Error::Io(ref err) => Some(err),
      Error::Regex(ref err) => Some(err),
    }
  }
}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Error {
    Error::Io(err)
  }
}

impl From<regex::Error> for Error {
  fn from(err: regex::Error) -> Error {
    Error::Regex(err)
  }
}


fn main() {
  // nc -lU /tmp/scdaemon.sock

  match main_loop("/tmp/scdaemon.sock", "/tmp/scdaemon.log") {
    Ok(_) => exit(0),
    Err(err) => {
      write!(io::stderr(), "{}", err).unwrap();
      exit(1)
    }
  }
}

fn main_loop(socket_path: &str, log_path: &str) -> Result<String, Error> {
  let socket = UnixStream::connect(socket_path)?;
  let mut log = File::create(log_path)?;
  let lines = BufReader::new(socket).lines();

  let match_set = RegexSet::new(&[r"PK(SIGN|AUTH)", r"result: Success", r"result: "])?;
  let mut notes = Vec::new();

  // | while read line; do
  for line_result in lines {
    let line = line_result?;
    write!(log, "{}", line).ok();
    let matches = match_set.matches(&line);
    if matches.matched(0) {
      let note = Notification::new()
        .summary("GPG activity")
        .body("A process is waiting on the Yubikey")
        .appname("scdaemon")
        .timeout(30000)
        .finalize();

      match note.show() {
        Ok(handle) => notes.push(handle),
        Err(err) => write!(log, "Problem notifying about socket message: {}", err).unwrap_or(()),
      };
      continue;
    }
    if matches.matched(1) {
      write!(log, "Recieved success report on socket, notifying").ok();
      match notes.pop() {
        Some(ref mut handle) => {
          handle.body("Accepted!").timeout(200);
          handle.update()
        }
        None => write!(log, "No matching notification to update.").unwrap_or(()),
      }
      continue;
    }
    if matches.matched(2) {
      write!(log, "Recieved failure report on socket, notifying").ok();
      match notes.pop() {
        Some(ref mut handle) => {
          handle.body("FAILED!").timeout(500);
          handle.update()
        }
        None => write!(log, "No matching notification to update.").unwrap_or(()),
      }
      continue;
    }
  }

  return Ok("done".to_string());

  // if echo $line | egrep -q 'PK(SIGN|AUTH)'; then
  // notify-send "GPG activity" "A process is waiting on the Yubikey!"
  // echo "Notifying" >> /tmp/scdaemon.log
  // fi
  // echo $line >> /tmp/scdaemon.log
  // done
  //
}
