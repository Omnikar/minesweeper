#[allow(dead_code)]
mod board;

use board::Board;
use std::io::{self, BufWriter, Write};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let stdin = io::stdin();
    let mut stdout = BufWriter::new(io::stdout().into_raw_mode()?);

    macro_rules! run_diff {
        ($diff:ident) => {
            run(crate::board::difficulty::$diff::blank(), stdout)
        };
    }

    let diffs = ["Beginner", "Intermediate", "Expert"];
    let mut cursor_pos: usize = 0;

    macro_rules! render {
        () => {
            write!(
                stdout,
                "{}{}",
                termion::clear::All,
                termion::cursor::Goto(1, 1),
            )?;
            for (i, &diff) in diffs.iter().enumerate()
            {
                if cursor_pos == i
                {
                    write!(stdout, "\x1b[96m>\x1b[0m {}\n\r", diff)?;
                }
                else
                {
                    write!(stdout, "  {}\n\r", diff)?;
                }
            }
        };
    }

    write!(stdout, "{}", termion::cursor::Hide)?;
    render!();
    stdout.flush()?;

    for k in stdin.keys()
    {
        match k?
        {
            Key::Ctrl('c') =>
            {
                write!(stdout, "{}", termion::cursor::Show)?;
                return Ok(());
            }
            Key::Char('j') | Key::Down => cursor_pos = (cursor_pos + 1).min(diffs.len() - 1),
            Key::Char('k') | Key::Up =>
            {
                cursor_pos = cursor_pos.checked_sub(1).unwrap_or(cursor_pos)
            }
            Key::Char('d') | Key::Char(' ') =>
            {
                return match cursor_pos
                {
                    0 => run_diff!(Beginner),
                    1 => run_diff!(Medium),
                    2 => run_diff!(Expert),
                    _ => panic!("This is a bug."),
                }
            }
            _ => continue,
        }

        render!();
        stdout.flush()?;
    }

    run_diff!(Beginner)
}

fn run<const ROWS: usize, const COLS: usize, const MINES: usize>(
    mut board: Board<ROWS, COLS, MINES>,
    mut stdout: impl Write,
) -> Result<(), Box<dyn std::error::Error>>
{
    write!(stdout, "{}", termion::cursor::Show)?;

    let mut csr_pos: (u16, u16) = (1, 1);

    macro_rules! render {
        (board) => {
            write!(
                stdout,
                "{}{}",
                termion::clear::All,
                termion::cursor::Goto(1, 1),
            )?;
            board.draw(&mut stdout)?;
            write!(stdout, "\n\x1b[96m\x1b[s?\x1b[0m\r{} flags left", board.flags_left())?;
        };
        (help) => {
            write!(
                stdout,
                "{}{}",
                termion::clear::All,
                termion::cursor::Goto(1, 1)
            )?;
            write!(
                stdout,
                "\u{2190}\u{2193}\u{2191}\u{2192}/hjkl - move cursor\n\r"
            )?;
            write!(stdout, "        d - dig\n\r")?;
            write!(stdout, "        f - flag\n\r")?;
            stdout.flush()?;
            continue;
        };
    }

    render!(board);
    write!(
        stdout,
        "{}",
        termion::cursor::Goto(csr_pos.0 * 4 - 1, csr_pos.1 * 2)
    )?;
    stdout.flush()?;

    let mut generated = false;

    let stdin = io::stdin();

    for k in stdin.keys()
    {
        match k?
        {
            Key::Ctrl('c') =>
            {
                render!(board);
                write!(stdout, "\n\r")?;
                break;
            }
            Key::Char('l') | Key::Right =>
            {
                csr_pos.0 = (csr_pos.0 + 1).clamp(1, COLS as u16)
            }
            Key::Char('h') | Key::Left =>
            {
                csr_pos.0 = (csr_pos.0 - 1).clamp(1, COLS as u16)
            }
            Key::Char('j') | Key::Down =>
            {
                csr_pos.1 = (csr_pos.1 + 1).clamp(1, (ROWS as u16) + 1)
            }
            Key::Char('k') | Key::Up =>
            {
                csr_pos.1 = (csr_pos.1 - 1).clamp(1, (ROWS as u16) + 1)
            }
            Key::Char('d') | Key::Char(' ') =>
            {
                if csr_pos.1 > ROWS as u16
                {
                    render!(help);
                }
                if !generated
                {
                    let b_pos = ((csr_pos.1 - 1) as usize, (csr_pos.0 - 1) as usize);
                    let mut avoid = Board::<ROWS, COLS, MINES>::adjs(b_pos.0, b_pos.1);
                    avoid.push(b_pos);
                    board.randomize(&avoid);
                    board.set_nums();
                    generated = true;
                }
                if board.open((csr_pos.1 - 1) as usize, (csr_pos.0 - 1) as usize)
                {
                    board.reveal();
                    render!(board);
                    write!(stdout, "\n\r")?;
                    stdout.flush()?;
                    break;
                }
            }
            Key::Char('f') | Key::Char('\t') =>
            {
                if csr_pos.1 <= ROWS as u16
                {
                    board.toggle_flag((csr_pos.1 - 1) as usize, (csr_pos.0 - 1) as usize)
                }
            }
            Key::Char('?') =>
            {
                render!(help);
            }
            _ => continue,
        }

        if board.spaces_left() == MINES
        {
            board.flag_all();
        }
        render!(board);
        if board.spaces_left() == MINES
        {
            write!(stdout, "\n\r")?;
            stdout.flush()?;
            break;
        }
        if csr_pos.1 <= ROWS as u16
        {
            write!(
                stdout,
                "{}",
                termion::cursor::Goto(csr_pos.0 * 4 - 1, csr_pos.1 * 2)
            )?;
        }
        else
        {
            write!(stdout, "\x1b[u")?;
        }
        stdout.flush()?;
    }

    write!(stdout, "{}", termion::cursor::Show)?;
    stdout.flush()?;
    Ok(())
}
