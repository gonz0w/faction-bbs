//! Disposable dependency probe; not a BBS or a deployable authentication service.
#![forbid(unsafe_code)]
use mio::{Events, Interest, Poll, Token};
use rusqlite::{Connection, OpenFlags, params};
use std::{error::Error, fs, io::{self, BufRead, Read, Write}, net::{TcpListener, TcpStream},
    os::unix::net::{UnixListener, UnixStream}, path::Path, time::{Duration, Instant}};
type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;
fn db(path: &str) -> Result<Connection> {
    let c = Connection::open(path)?;
    c.busy_timeout(Duration::from_millis(100))?;
    c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;
        PRAGMA cache_size=-2048; PRAGMA mmap_size=0;
        CREATE TABLE IF NOT EXISTS members(id INTEGER PRIMARY KEY, approved INTEGER NOT NULL CHECK(approved IN(0,1)));
        CREATE TABLE IF NOT EXISTS posts(id INTEGER PRIMARY KEY, author INTEGER NOT NULL REFERENCES members(id), body TEXT NOT NULL);
        CREATE VIRTUAL TABLE IF NOT EXISTS search USING fts5(body);
        INSERT OR IGNORE INTO members VALUES(1,0);")?;
    Ok(c)
}
fn approved(c: &Connection) -> rusqlite::Result<bool> {
    c.query_row("SELECT approved FROM members WHERE id=1", [], |r| r.get(0))
}
fn action(c: &mut Connection, command: &[u8]) -> Result<&'static str> {
    let t = c.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    if !approved(&t)? { return Ok("DENIED"); }
    let answer = match command {
        b"READ" => { let _: i64 = t.query_row("SELECT count(*) FROM posts", [], |r| r.get(0))?; "ROOM" }
        b"POST" => { t.execute("INSERT INTO posts(author,body) VALUES(1,?1)", ["synthetic post"])?; "POSTED" }
        _ => "UNKNOWN",
    };
    t.commit()?;
    Ok(answer)
}
fn bounded_line(reader: &mut impl BufRead) -> Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    let n = reader.take(1026).read_until(b'\n', &mut line)?;
    if n == 0 { return Ok(None); }
    if n > 1025 || line.last() != Some(&b'\n') { return Err("oversized or unterminated command".into()); }
    line.pop();
    if line.last() == Some(&b'\r') { line.pop(); }
    if line.len() > 1024 { return Err("oversized command".into()); }
    Ok(Some(line))
}
fn serve(path: &str) -> Result {
    if std::env::var_os("SSH_ORIGINAL_COMMAND").is_some_and(|v| !v.is_empty()) {
        return Err("exec/subsystem denied".into());
    }
    // No shell commands, environment interpolation, user-supplied paths or content.
    let mut c = db(path)?;
    let stdin = io::stdin(); let mut reader = stdin.lock();
    let mut out = io::stdout().lock();
    writeln!(out, "FACTION DEPENDENCY PROBE")?; out.flush()?;
    while let Some(line) = bounded_line(&mut reader)? {
        if line == b"QUIT" { break; }
        writeln!(out, "{}", action(&mut c, &line)?)?; out.flush()?;
    }
    Ok(())
}
fn sockets(dir: &Path) -> Result {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
    let mut client = TcpStream::connect(listener.local_addr()?)?;
    let (server, _) = listener.accept()?; server.set_nonblocking(true)?;
    let mut server = mio::net::TcpStream::from_std(server);
    let mut poll = Poll::new()?; let mut events = Events::with_capacity(8);
    poll.registry().register(&mut server, Token(1), Interest::READABLE)?;
    client.write_all(b"READ\n")?;
    poll.poll(&mut events, Some(Duration::from_secs(2)))?;
    assert!(events.iter().any(|e| e.token() == Token(1) && e.is_readable()));
    let mut b = [0; 8]; assert_eq!(server.read(&mut b)?, 5); assert_eq!(&b[..5], b"READ\n");
    // A stalled peer must not prevent a second ready peer being serviced.
    let _stalled = TcpStream::connect(listener.local_addr()?)?;
    let (stalled, _) = listener.accept()?; stalled.set_nonblocking(true)?;
    let mut stalled = mio::net::TcpStream::from_std(stalled);
    poll.registry().register(&mut stalled, Token(2), Interest::READABLE)?;
    client.write_all(b"POST\n")?;
    poll.poll(&mut events, Some(Duration::from_secs(2)))?;
    assert!(events.iter().any(|e| e.token() == Token(1) && e.is_readable()));
    assert_eq!(server.read(&mut b)?, 5);
    let socket = dir.join("probe.sock"); let listener = UnixListener::bind(&socket)?;
    let mut client = UnixStream::connect(&socket)?;
    let (server, _) = listener.accept()?; server.set_nonblocking(true)?;
    let mut server = mio::net::UnixStream::from_std(server);
    poll.registry().register(&mut server, Token(3), Interest::READABLE)?;
    client.write_all(b"LOCAL\n")?;
    poll.poll(&mut events, Some(Duration::from_secs(2)))?;
    assert!(events.iter().any(|e| e.token() == Token(3) && e.is_readable()));
    assert_eq!(server.read(&mut b)?, 6); fs::remove_file(socket)?;
    assert!(bounded_line(&mut io::Cursor::new(vec![b'x'; 2048])).is_err());
    assert_eq!(bounded_line(&mut io::Cursor::new(b"READ\r\n"))?, Some(b"READ".to_vec()));
    println!("PASS tcp_readiness unix_readiness stalled_peer bounded_input");
    Ok(())
}
fn selftest(dir: &Path) -> Result {
    fs::create_dir_all(dir)?; sockets(dir)?;
    let path = dir.join("probe.db"); let path = path.to_str().ok_or("path")?;
    let mut c = db(path)?;
    assert_eq!(action(&mut c, b"READ")?, "DENIED");
    c.execute("UPDATE members SET approved=1 WHERE id=1", [])?;
    assert_eq!(action(&mut c, b"POST")?, "POSTED");
    c.execute("UPDATE members SET approved=0 WHERE id=1", [])?;
    assert_eq!(action(&mut c, b"POST")?, "DENIED");
    assert!(c.execute("INSERT INTO posts(author,body) VALUES(99,'bad')", []).is_err());
    let text = "parameter ' ; DROP TABLE posts; --";
    c.execute("INSERT INTO posts(author,body) VALUES(1,?1)", [text])?;
    let found: String = c.query_row("SELECT body FROM posts WHERE body=?1", [text], |r| r.get(0))?;
    assert_eq!(found,text);
    c.execute("INSERT INTO search(body) VALUES(?1)", [text])?;
    let n:i64=c.query_row("SELECT count(*) FROM search WHERE search MATCH 'parameter'",[],|r|r.get(0))?;
    assert_eq!(n,1);
    let journal:String=c.query_row("PRAGMA journal_mode",[],|r|r.get(0))?; assert_eq!(journal,"wal");
    let sync:i64=c.query_row("PRAGMA synchronous",[],|r|r.get(0))?; assert_eq!(sync,2);
    let second=db(path)?; c.execute_batch("BEGIN IMMEDIATE")?;
    let start=Instant::now();
    let err=second.execute("INSERT INTO posts(author,body) VALUES(1,'busy')",[]).unwrap_err();
    assert_eq!(err.sqlite_error_code(),Some(rusqlite::ErrorCode::DatabaseBusy));
    let busy_ms=start.elapsed().as_millis(); assert!(busy_ms<2000);
    c.execute_batch("ROLLBACK")?;
    // Backup while live; restore through a new connection and compare records.
    let backup=dir.join("backup.db");
    c.backup("main", &backup, None)?;
    let restored=Connection::open_with_flags(&backup,OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let integrity:String=restored.query_row("PRAGMA integrity_check",[],|r|r.get(0))?; assert_eq!(integrity,"ok");
    let records:Vec<(i64,String)>=c.prepare("SELECT author,body FROM posts ORDER BY id")?
        .query_map([],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<std::result::Result<_,_>>()?;
    // Explicit domain-record export/import; binary strings are length-prefixed UTF-8.
    let mut export=Vec::new();
    for (author,body) in &records {
        export.extend(author.to_le_bytes()); export.extend((body.len() as u64).to_le_bytes()); export.extend(body.as_bytes());
    }
    fs::write(dir.join("records.bin"),&export)?;
    let imported=db(dir.join("import.db").to_str().ok_or("path")?)?;
    let mut input=io::Cursor::new(export);
    while input.position() < input.get_ref().len() as u64 {
        let mut b=[0;8]; input.read_exact(&mut b)?; let author=i64::from_le_bytes(b);
        input.read_exact(&mut b)?; let len=u64::from_le_bytes(b); assert!(len<=1024);
        let mut body=vec![0;len as usize]; input.read_exact(&mut body)?;
        imported.execute("INSERT INTO posts(author,body) VALUES(?1,?2)",params![author,String::from_utf8(body)?])?;
    }
    for other in [&restored,&imported] {
        let actual:Vec<(i64,String)>=other.prepare("SELECT author,body FROM posts ORDER BY id")?
            .query_map([],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<std::result::Result<_,_>>()?;
        assert_eq!(actual,records);
    }
    println!("PASS sqlite={} wal_full foreign_keys parameters fts5 busy_timeout_ms={} approval backup record_roundtrip",rusqlite::version(),busy_ms);
    Ok(())
}
fn main() -> Result {
    let args:Vec<String>=std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("selftest") if args.len()==3 => selftest(Path::new(&args[2])),
        Some("init") if args.len()==3 => { db(&args[2])?; Ok(()) },
        Some("approval") if args.len()==4 => { let c=db(&args[2])?; let value:u8=args[3].parse()?;
            if value>1 { return Err("approval must be 0 or 1".into()); }
            c.execute("UPDATE members SET approved=?1 WHERE id=1",[value])?; Ok(()) },
        Some("serve") if args.len()==3 => serve(&args[2]),
        _=>Err("usage: selftest DIR | init DB | approval DB 0/1 | serve DB".into()),
    }
}
