#!/usr/bin/env python3
"""Reuse the unchanged frontend oracles against the sealed setuid binary.

Only paths change: private mock sockets live in the shared nullfs run directory.
The original actual-TTY/peer/status/privilege assertions remain intact.
"""
from pathlib import Path
source=Path(__file__).resolve().parents[1]/'055-confinement/frontend_probe.py'
text=source.read_text().replace("binary='/usr/local/libexec/faction-tty'", "binary='/usr/local/libexec/faction-isolated/faction-tty'")
text=text.replace("dir='/var/run'", "dir='/var/chroot/faction/var/run/faction'")
text=text.replace("['--socket',str(path)", "['--socket',str(path).removeprefix('/var/chroot/faction')")
text=text.replace("['--socket',str(link)", "['--socket',str(link).removeprefix('/var/chroot/faction')")
exec(compile(text,str(source),'exec'),{'__name__':'__main__'})
