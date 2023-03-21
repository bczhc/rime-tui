rime-tui
--

A TUI App for Rime.

![gif](https://raw.githubusercontent.com/bczhc/rime-tui/master/img/screenrecord.gif)

## Usage:

<pre><u style="text-decoration-style:single"><b>Usage:</b></u> <b>rime-tui</b> [OPTIONS]

<u style="text-decoration-style:single"><b>Options:</b></u>
  <b>-s</b>, <b>--schema</b> &lt;schema&gt;              
      <b>--shared-dir</b> &lt;shared-dir&gt;      Rime shared data directory [default: /usr/share/rime-data/]
      <b>--user-dir</b> &lt;user-dir&gt;          Rime user data directory [default: /home/bczhc/.local/share/fcitx5/rime]
      <b>--exit-command</b> &lt;exit-command&gt;  Input command for exiting the program [default: /exit]
      <b>--copy-command</b> &lt;copy-command&gt;  Input command for putting the output into X11 clipboard [default: /copy]
      <b>--load-command</b> &lt;load-command&gt;  Input command for loading from X11 clipboard [default: /load]
  <b>-h</b>, <b>--help</b>                         Print help</pre>

Currently, this program only runs on *nix
operating systems, with X11 graphics environment.
The reason writes below:

Pure terminal has no way to listen key
down events (say, raw key events), and for this program key
listening is achieved via X11 APIs. This means
an active X11 server is also needed.

Seems librime doesn't provide a way to redirect
its log outputs, so I use "file descriptor duplication"
(Rust `gag` crate)
and `pipe(2)` to intercept stderr, printing
its content inside the TUI App "Output" area. These APIs
only exist on *nix platforms.