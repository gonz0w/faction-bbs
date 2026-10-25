use faction_omemo_proof::{MAX_LINE, dispatch, require_private_pipes, wipe};
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    // Private local research pipes only. No secret argv/env or filesystem route.
    if std::env::args_os().len() != 1 {
        return Err(io::Error::other("no arguments accepted"));
    }
    require_private_pipes().map_err(|_| io::Error::other("private input/output pipes required"))?;
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    let mut state = None;
    loop {
        let mut line = Vec::new();
        let mut oversized = false;
        let mut eof = false;
        // Consume/drain a line incrementally; never allocate its unbounded length.
        loop {
            let available = input.fill_buf()?;
            if available.is_empty() {
                eof = true;
                break;
            }
            let newline = available.iter().position(|b| *b == b'\n');
            let take = newline.map_or(available.len(), |n| n + 1);
            if !oversized {
                if line.len() + take > MAX_LINE + 1 {
                    oversized = true;
                    wipe(&mut line);
                    line.clear();
                } else {
                    line.extend_from_slice(&available[..take]);
                }
            }
            input.consume(take);
            if newline.is_some() {
                break;
            }
        }
        if eof && line.is_empty() && !oversized {
            break;
        }
        let mut reply = if oversized || eof {
            "err".to_owned()
        } else {
            line.pop();
            match std::str::from_utf8(&line) {
                Ok(text) => dispatch(&mut state, text),
                Err(_) => "err".into(),
            }
        }
        .into_bytes();
        wipe(&mut line);
        let result = output
            .write_all(&reply)
            .and_then(|_| output.write_all(b"\n"))
            .and_then(|_| output.flush());
        wipe(&mut reply);
        result?;
        if eof {
            break;
        }
    }
    Ok(())
}
