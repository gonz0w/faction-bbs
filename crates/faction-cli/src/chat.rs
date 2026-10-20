//! Local console chat, not an IRC protocol endpoint or remote-login entry point.
use crate::{Result, bounded_line, safe, terminal};
use faction_core::{Board, ChatCommand, ChatKind, ChatLease, ChatPage, Error, Session};
use faction_terminal::{LINE_BYTES, LineEditor, LineEvent};
use std::{
    fs::OpenOptions,
    io::{self, Write},
    time::{Duration, Instant},
};
struct State {
    channel: Option<String>,
    cursor: i64,
    request: i64,
}
impl State {
    fn name(&self) -> faction_core::Result<&str> {
        self.channel
            .as_deref()
            .ok_or(Error::Invalid("join a channel first"))
    }
    fn label(&self) -> String {
        format!("chat[{}]> ", self.channel.as_deref().unwrap_or("-"))
    }
}
fn page_text(page: ChatPage) -> String {
    let mut out = String::new();
    if page.gap {
        out.push_str("History gap: older chat expired; showing retained messages.\n")
    }
    for m in page.messages {
        out.push_str(&format!(
            "{} #{} {}{}{}\n",
            if m.kind == ChatKind::Emote { "*" } else { "<" },
            m.sequence,
            safe(&m.author),
            if m.kind == ChatKind::Emote { " " } else { "> " },
            safe(&m.body)
        ));
    }
    if page.more {
        out.push_str(&format!(
            "More retained history: /history {}\n",
            page.cursor
        ));
    }
    out
}
fn poll_messages(
    board: &mut Board,
    lease: &ChatLease,
    state: &mut State,
    after: Option<i64>,
) -> faction_core::Result<String> {
    let Some(channel) = &state.channel else {
        return Ok(String::new());
    };
    let page = board.chat_read(lease, channel, after.unwrap_or(state.cursor))?;
    state.cursor = state.cursor.max(page.cursor);
    Ok(page_text(page))
}
fn execute(
    board: &mut Board,
    s: &Session,
    lease: &ChatLease,
    state: &mut State,
    input: &str,
) -> faction_core::Result<(bool, String)> {
    let command = if input.trim().is_empty() {
        ChatCommand::Poll
    } else {
        ChatCommand::parse(input)?
    };
    let mut out = String::new();
    match command {
        ChatCommand::Help=>out.push_str("/list /join #channel /part /names [after-handle] /history [after-sequence] /poll /me text /quit\nPlain text sends to the selected channel; //text sends a literal slash.\nSysop: /create #channel board|private, /allow #channel handle, /deny #channel handle, /kick #channel handle\nAll channels are local; retained history begins at your join.\n"),
        ChatCommand::List=>{for c in board.chat_channels(lease)?{out.push_str(&format!("{} ({}, local)\n",safe(&c.name),if c.private{"private"}else{"board"}));}},
        ChatCommand::Join(name)=>{let c=board.chat_join(lease,name)?;state.cursor=c.sequence;state.channel=Some(c.name);out.push_str("Joined; new messages start here. /history replays eligible retained chat.\n");},
        ChatCommand::Part=>{board.chat_part(lease,state.name()?)?;state.channel=None;state.cursor=0;out.push_str("Left channel.\n");},
        ChatCommand::Names(after)=>{let names=board.chat_names(lease,state.name()?,after)?;out.push_str(&format!("Members: {}\n",names.handles.iter().map(|h|safe(h)).collect::<Vec<_>>().join(" ")));if let Some(next)=names.next{out.push_str(&format!("More: /names {next}\n"));}},
        ChatCommand::History(after)=>{out.push_str(&poll_messages(board,lease,state,Some(after))?);if out.is_empty(){out.push_str("No retained messages since this join.\n");}},
        ChatCommand::Poll=>{out.push_str(&poll_messages(board,lease,state,None)?);if out.is_empty(){out.push_str("No new chat messages.\n");}},
        ChatCommand::Say(body)|ChatCommand::Emote(body)=>{
            let kind=if matches!(command,ChatCommand::Emote(_)){ChatKind::Emote}else{ChatKind::Say};
            board.chat_send(lease,state.name()?,state.request,body,kind)?;
            state.request=state.request.checked_add(1).ok_or(Error::Invalid("request sequence exhausted"))?;
        },
        ChatCommand::Create(name,private)=>{board.chat_create(s,name,private)?;out.push_str("Channel created.\n");},
        ChatCommand::Allow(name,member,allow)=>{board.chat_allow(s,name,member,allow)?;out.push_str("Channel grant updated; target must join again.\n");},
        ChatCommand::Kick(name,member)=>{board.chat_kick(s,name,member)?;out.push_str("Member parted; a kick does not remove a private grant.\n");},
        ChatCommand::Quit=>return Ok((false,"Chat closed.\n".into())),
    }
    Ok((true, out))
}
fn recover(error: Error, state: &mut State, editor: Option<&mut LineEditor>) -> Result<String> {
    match error {
        Error::StaleSession
        | Error::UnsafeState
        | Error::UnsupportedSchema
        | Error::Io(_)
        | Error::Entropy
        | Error::PasswordHash => Err(error.into()),
        Error::Denied => {
            state.channel = None;
            state.cursor = 0;
            if let Some(editor) = editor {
                editor.clear();
            }
            Ok("Channel access changed or action denied; draft discarded. Join explicitly before sending.\n".into())
        }
        _ => Ok(format!(
            "Chat: {error}. A failed send keeps its request sequence; retry the exact text.\n"
        )),
    }
}
fn heartbeat(board: &mut Board, lease: &ChatLease, last: &mut Instant) -> Result<()> {
    if last.elapsed() >= Duration::from_secs(10) {
        match board.chat_heartbeat(lease) {
            Ok(()) => *last = Instant::now(),
            Err(Error::Busy) => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
fn plain(board: &mut Board, s: &Session, lease: &ChatLease, state: &mut State) -> Result<()> {
    println!("Plain chat: Enter submits; /poll or blank line reads new messages.");
    let mut tty = OpenOptions::new().read(true).open("/dev/tty")?;
    let mut last = Instant::now();
    loop {
        print!("{}", state.label());
        io::stdout().flush()?;
        loop {
            heartbeat(board, lease, &mut last)?;
            if terminal::ready()? {
                break;
            }
        }
        let line = bounded_line(&mut tty, LINE_BYTES)?;
        let fetch = match execute(board, s, lease, state, &line) {
            Ok((running, out)) => {
                print!("{out}");
                if !running {
                    return Ok(());
                }
                out.is_empty()
            }
            Err(e) => {
                print!("{}", recover(e, state, None)?);
                false
            }
        };
        // Read commands already fetched a page; only sends need a fresh echo page.
        if fetch {
            match poll_messages(board, lease, state, None) {
                Ok(out) => print!("{out}"),
                Err(e) => print!("{}", recover(e, state, None)?),
            }
        }
    }
}
fn redraw(
    output: &mut impl Write,
    state: &State,
    editor: &LineEditor,
    columns: u16,
) -> io::Result<()> {
    let label = state.label();
    let width = (columns as usize).saturating_sub(label.len() + 2);
    write!(output, "\r\x1b[2K{label}{}", editor.viewport(width))?;
    output.flush()
}
fn notify(output: &mut impl Write, text: &str) -> io::Result<()> {
    if !text.is_empty() {
        write!(output, "\r\x1b[2K{text}")?;
    }
    Ok(())
}
fn live(board: &mut Board, s: &Session, lease: &ChatLease, state: &mut State) -> Result<()> {
    let raw = terminal::Raw::open()?;
    let mut editor = LineEditor::default();
    let mut output = io::stdout();
    let mut last = Instant::now();
    let mut received = Instant::now();
    let mut dimensions =
        terminal::size().ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))?;
    output
        .write_all(b"Live chat: Enter sends, Ctrl-C/D returns; bounded ASCII editor, /help.\r\n")?;
    redraw(&mut output, state, &editor, dimensions.0)?;
    loop {
        heartbeat(board, lease, &mut last)?;
        if !terminal::capable() {
            notify(
                &mut output,
                &format!(
                    "Chat closed after resize; unsent draft was not submitted: {}\n",
                    editor.text()
                ),
            )?;
            return Ok(());
        }
        let current_size =
            terminal::size().ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))?;
        if dimensions != current_size {
            dimensions = current_size;
            redraw(&mut output, state, &editor, dimensions.0)?;
        }
        let ready = terminal::ready()?;
        let event = if ready {
            editor.feed(raw.read_byte()?, Instant::now())
        } else {
            editor.expire(Instant::now())
        };
        let mut dirty = ready || event.is_some();
        if let Some(event) = event {
            match event {
                LineEvent::Cancel => {
                    notify(&mut output, "Chat closed.\n")?;
                    return Ok(());
                }
                LineEvent::Discarded => notify(
                    &mut output,
                    "Input discarded; press Enter after an unknown escape sequence.\n",
                )?,
                LineEvent::Submit(input) => {
                    output.write_all(b"\r\n")?;
                    match execute(board, s, lease, state, &input) {
                        Ok((running, out)) => {
                            notify(&mut output, &out)?;
                            if !running {
                                return Ok(());
                            }
                        }
                        Err(e) => notify(&mut output, &recover(e, state, Some(&mut editor))?)?,
                    }
                    received = Instant::now() - Duration::from_secs(1);
                }
            }
        }
        if received.elapsed() >= Duration::from_millis(250) {
            match poll_messages(board, lease, state, None) {
                Ok(out) => {
                    dirty |= !out.is_empty();
                    notify(&mut output, &out)?
                }
                Err(Error::Busy) => (),
                Err(e) => {
                    dirty = true;
                    notify(&mut output, &recover(e, state, Some(&mut editor))?)?
                }
            }
            received = Instant::now();
        }
        if dirty {
            redraw(&mut output, state, &editor, dimensions.0)?;
        }
    }
}
pub fn run(board: &mut Board, s: &Session, ansi: bool) -> Result<()> {
    let lease = board.chat_open(s)?;
    let outcome = (|| -> Result<()> {
        let lobby = board.chat_join(&lease, "#lobby")?;
        let mut state = State {
            channel: Some(lobby.name),
            cursor: lobby.sequence,
            request: 1,
        };
        println!("BBS-local chat. No federation or external IRC connection. /help for commands.");
        if ansi && terminal::capable() {
            live(board, s, &lease, &mut state)
        } else {
            plain(board, s, &lease, &mut state)
        }
    })();
    let closed = board.chat_close(&lease);
    outcome?;
    closed?;
    Ok(())
}
