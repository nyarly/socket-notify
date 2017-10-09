extern crate regex;
extern crate notify_rust;

use std::os::unix::net::UnixStream;
use std::io::Write;
use std::fs::File;
use std::io::{BufRead, BufReader};

use regex::RegexSet;
use notify_rust::{Notification, NotificationHandle};

fn main() {
    // nc -lU /tmp/scdaemon.sock
    let mut socket = UnixStream::connect("/tmp/scdaemon.sock")?;
    let mut log = File::create("/tmp/scdaemon.log")?;
    let mut lines = BufReader::new(socket).lines();

    let match_set = RegexSet::new(&[r"PK(SIGN|AUTH)", r"result: Success", r"result: "])?;
    let mut notes = Vec<NotificationHandle>;

    // | while read line; do
    for line in lines {
        write!(log, "{}", line);
        let matches = match_set.matches(line);
        if matches.matched(0) {
          let note = Notification::new()
            .summary("GPG activity")
            .body("A process is waiting on the Yubikey")
            .appname("scdaemon");

          match note.show() {
            Ok(handle) => notes.push(handle),
            Err(err) => write!(log, "Problem notifying about socket message: {}", err),
          }

        }
        if matches.matched(1) {
          write!(log, "Recieved success report on socket, notifying");
          continue;
        }
        if matches.matched(2) {
          write!(log, "Recieved failure report on socket, notifying");
          continue;
        }
    }

    // if echo $line | egrep -q 'PK(SIGN|AUTH)'; then
    // notify-send "GPG activity" "A process is waiting on the Yubikey!"
    // echo "Notifying" >> /tmp/scdaemon.log
    // fi
    // echo $line >> /tmp/scdaemon.log
    // done
    //
}
