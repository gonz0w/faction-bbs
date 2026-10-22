use crate::{Result, Style};
use std::fmt::Write;
const WIDTH: usize = 80;
const HEIGHT: usize = 16;
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Cell {
    glyph: char,
    style: Style,
}
/// A fixed 80x16 cell image. The original bytes never reach the terminal.
#[derive(Clone)]
pub struct Art {
    cells: Vec<Cell>,
}
impl Art {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 16384 {
            return Err("art exceeds 16 KiB");
        }
        let (bytes, ice) = sauce_payload(bytes)?;
        let mut bright_background = false;
        let blank = Cell {
            glyph: ' ',
            style: Style::default(),
        };
        let mut art = Self {
            cells: vec![blank; WIDTH * HEIGHT],
        };
        let (mut x, mut y, mut style, mut saved, mut at) =
            (0usize, 0usize, Style::default(), (0, 0), 0usize);
        while at < bytes.len() {
            let b = bytes[at];
            at += 1;
            match b {
                27 => {
                    if bytes.get(at) != Some(&b'[') {
                        return Err("art allows only bounded CSI controls");
                    };
                    at += 1;
                    let start = at;
                    while bytes
                        .get(at)
                        .is_some_and(|b| b.is_ascii_digit() || *b == b';')
                    {
                        at += 1;
                        if at - start > 32 {
                            return Err("art control too long");
                        }
                    }
                    let command = *bytes.get(at).ok_or("incomplete art control")?;
                    at += 1;
                    let fields = std::str::from_utf8(&bytes[start..at - 1])
                        .map_err(|_| "invalid art parameters")?;
                    let values: Vec<usize> = if fields.is_empty() {
                        vec![]
                    } else {
                        fields
                            .split(';')
                            .map(|v| {
                                if v.is_empty() {
                                    Ok(0)
                                } else {
                                    v.parse().map_err(|_| "invalid art number")
                                }
                            })
                            .collect::<Result<_>>()?
                    };
                    let one = values.first().copied().unwrap_or(1).max(1);
                    match command {
                        b'm' => {
                            if values.is_empty() {
                                style = Style::default();
                                bright_background = false;
                            } else {
                                for value in values {
                                    match value {
                                        0 => {
                                            style = Style::default();
                                            bright_background = false;
                                        }
                                        1 => style.bold = true,
                                        22 => style.bold = false,
                                        30..=37 => style.fg = (value - 30) as u8,
                                        39 => style.fg = 7,
                                        5 if ice => {
                                            bright_background = true;
                                            style.bg |= 8;
                                        }
                                        25 if ice => {
                                            bright_background = false;
                                            style.bg &= 7;
                                        }
                                        40..=47 => {
                                            style.bg = (value - 40) as u8
                                                | if bright_background { 8 } else { 0 }
                                        }
                                        49 => style.bg = if bright_background { 8 } else { 0 },
                                        90..=97 => style.fg = (value - 90 + 8) as u8,
                                        100..=107 => style.bg = (value - 100 + 8) as u8,
                                        _ => {
                                            return Err(
                                                "unsupported art SGR (blink requires SAUCE non-blink mode)",
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        b'H' | b'f' if values.len() <= 2 => {
                            y = values.first().copied().unwrap_or(1).max(1) - 1;
                            x = values.get(1).copied().unwrap_or(1).max(1) - 1;
                        }
                        b'A' if values.len() <= 1 => {
                            y = y.checked_sub(one).ok_or("art cursor outside canvas")?
                        }
                        b'B' if values.len() <= 1 => {
                            y = y.checked_add(one).ok_or("art cursor overflow")?
                        }
                        b'C' if values.len() <= 1 => {
                            x = x.checked_add(one).ok_or("art cursor overflow")?
                        }
                        b'D' if values.len() <= 1 => {
                            x = x.checked_sub(one).ok_or("art cursor outside canvas")?
                        }
                        b's' if values.is_empty() => saved = (x, y),
                        b'u' if values.is_empty() => {
                            (x, y) = saved;
                        }
                        b'J' if values == [2] => {
                            art.cells.fill(Cell { glyph: ' ', style });
                        }
                        b'K' if values.is_empty() || values == [0] => {
                            if x >= WIDTH || y >= HEIGHT {
                                return Err("art cursor outside canvas");
                            }
                            art.cells[y * WIDTH + x..(y + 1) * WIDTH]
                                .fill(Cell { glyph: ' ', style });
                        }
                        _ => return Err("unsupported art control"),
                    }
                    if command != b'm' && (x >= WIDTH || y >= HEIGHT) {
                        return Err("art cursor outside canvas");
                    }
                }
                b'\r' => x = 0,
                b'\n' => {
                    x = 0;
                    y += 1;
                    if y > HEIGHT {
                        return Err("art too tall");
                    }
                }
                0..=31 => return Err("art contains unsupported controls or metadata"),
                _ => {
                    if x == WIDTH {
                        x = 0;
                        y += 1
                    }
                    if x >= WIDTH || y >= HEIGHT {
                        return Err("art exceeds canvas");
                    }
                    let glyph = if b == 127 {
                        '⌂'
                    } else if b >= 128 {
                        CP437
                            .chars()
                            .nth((b - 128) as usize)
                            .expect("complete CP437 mapping")
                    } else {
                        b as char
                    };
                    art.cells[y * WIDTH + x] = Cell { glyph, style };
                    x += 1;
                }
            }
        }
        if art.render().len() > 16384 {
            return Err("rendered art exceeds 16 KiB");
        }
        Ok(art)
    }
    pub fn render(&self) -> String {
        let mut out = String::new();
        let mut current = None;
        for (row, cells) in self.cells.chunks(WIDTH).enumerate() {
            let end = cells
                .iter()
                .rposition(|c| c.glyph != ' ' || c.style != Style::default())
                .map_or(0, |n| n + 1);
            if end == 0 {
                continue;
            }
            write!(out, "\x1b[{};1H", row + 1).expect("string output");
            for cell in &cells[..end] {
                if current != Some(cell.style) {
                    out.push_str(&cell.style.sgr());
                    current = Some(cell.style);
                }
                out.push(cell.glyph);
            }
        }
        out.push_str("\x1b[0m");
        out
    }
}
/// A deliberately small SAUCE00 Character/ANSi import profile. All metadata
/// stays off the terminal; record dimensions never enlarge the fixed canvas.
/// FileSize is advisory in revision 00.5, so locate EOF/comments from the tail.
fn sauce_payload(bytes: &[u8]) -> Result<(&[u8], bool)> {
    if bytes.len() < 128 || &bytes[bytes.len() - 128..bytes.len() - 123] != b"SAUCE" {
        return Ok((bytes, false));
    }
    let at = bytes.len() - 128;
    let record = &bytes[at..];
    if &record[5..7] != b"00" || record[94..96] != [1, 1] {
        return Err("only SAUCE00 Character/ANSi is supported");
    }
    let width = usize::from(u16::from_le_bytes([record[96], record[97]]));
    let height = usize::from(u16::from_le_bytes([record[98], record[99]]));
    if !matches!(width, 0 | WIDTH) || height > HEIGHT || record[105] & !0x1f != 0 {
        return Err("unsupported SAUCE dimensions or flags");
    }
    let comments = usize::from(record[104]);
    let end = if comments == 0 {
        at
    } else {
        let start = at
            .checked_sub(5 + comments * 64)
            .ok_or("truncated SAUCE comments")?;
        if &bytes[start..start + 5] != b"COMNT" {
            return Err("invalid SAUCE comments");
        }
        start
    };
    let eof = end.checked_sub(1).ok_or("missing SAUCE EOF")?;
    if bytes[eof] != 0x1a {
        return Err("SAUCE import requires EOF before metadata");
    }
    Ok((&bytes[..eof], record[105] & 1 != 0))
}

const CP437: &str = "ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■ ";
