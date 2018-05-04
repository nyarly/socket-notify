# Socket Notifier

Might be the most boring name for a project so far?

## Motivation

I use a Yubikey for GPG operations.
I sign my commits.
I use gpg-agent as an ssh agent.
It's a very convenient way to get
a reasonable level of security.

But, several times a day,
I go to commit code
or push a branch
and instead it times out
because I look away rather than
touching the blinky light
like I'm supposed to,
and my push times out.
I hate this,
not the least because it means
that my colleagues are left hanging on
a fix that never arrives.

For a while I had pre-commit hooks
and an SSH wrapper to remind me that
I might need to verify a signature,
but that led to
[problems.](https://github.com/golang/dep/issues/1209)
Also, there wasn't a way to do that
for signed tags,
which I also make pretty common use of.

So I started out using a bash script with netcat
viz:

```
#!/bin/sh

source /etc/profile

nc -lU /tmp/scdaemon.sock | while read line; do
  if echo $line | egrep -q 'PK(SIGN|AUTH)'; then
    notify-send "GPG activity" "A process is waiting on the Yubikey!"
    echo "Notifying" >> /tmp/scdaemon.log
  fi
  echo $line >> /tmp/scdaemon.log
done
```

but the notifications lingered after signing had occurred,
which meant I needed to invest attention to close them
or distinguish an old notification
from a new one that meant I needed to acknowledge it.

## Solution

Set up a `.gnupg/scdaemon.conf` like:

```
log-file socket:///tmp/scdaemon.sock
debug 1027
debug-assuan-log-cats 511
```

To get scdaemon to pick up the config,
you have to `gpg-agent-connect scd killscd`.
(or something like that - if you perfect the command, please PR the docs?)

Then `cargo install socket-notify`
and arrange for it to run while you're logged in.
For instance, I have

`~/.config/systemd/user/scdaemon-notify.service`:
```
[Unit]
Description=SmartCard Daemon Notifier
PartOf=graphical-session.target

[Service]
# The path will be different, because systemd requires absolute paths...
ExecStart=$(which socket-notify)

[Install]
WantedBy=graphical-session.target
```

..._et voila!_ You should receive notification like
"GPG Event"
whenever your verification of a signing or authentication operation is required.
When you approve the signature
(or it times out)
the notification is replaced with a short-lived update.

## Future Plans

(
if there's a call for it, or I feel the urge -
don't assume this'll happen on its own
)

In the present moment,
everything is hardcoded.
It'd be nice to specify a config file with
the socket location,
whether to log and where to log to
(currently, this always goes to /tmp/scdaemon.log),
patterns that trigger notifications and
patterns that "answer" a notification.
