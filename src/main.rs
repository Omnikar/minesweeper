#[allow(dead_code)]
mod board;

use board::{difficulty, BoardTrait};
use std::io::{self, BufWriter, Write};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let mut board = difficulty::Beginner::blank();
    let mut csr_pos: (u16, u16) = (1, 1);

    let stdin = io::stdin();
    let mut stdout = BufWriter::new(io::stdout().into_raw_mode()?);

    macro_rules! render {
        () => {
            write!(
                stdout,
                "{}{}",
                termion::clear::All,
                termion::cursor::Goto(1, 1),
            )?;
            board.draw(&mut stdout)?;
            write!(stdout, "\n\r{}", board.flags_left)?;
        };
    }

    render!();
    write!(
        stdout,
        "{}",
        termion::cursor::Goto(csr_pos.0 * 4 - 1, csr_pos.1 * 2)
    )?;
    stdout.flush()?;

    let mut generated = false;

    for c in stdin.keys()
    {
        match c?
        {
            Key::Ctrl('c') =>
            {
                render!();
                write!(stdout, "\n\r")?;
                break;
            }
            Key::Char('l') | Key::Right =>
            {
                csr_pos.0 = (csr_pos.0 + 1).clamp(1, board.columns() as u16)
            }
            Key::Char('h') | Key::Left =>
            {
                csr_pos.0 = (csr_pos.0 - 1).clamp(1, board.columns() as u16)
            }
            Key::Char('j') | Key::Down =>
            {
                csr_pos.1 = (csr_pos.1 + 1).clamp(1, board.rows() as u16)
            }
            Key::Char('k') | Key::Up =>
            {
                csr_pos.1 = (csr_pos.1 - 1).clamp(1, board.rows() as u16)
            }
            Key::Char('d') =>
            {
                if !generated
                {
                    board.randomize();
                    board.set_nums();
                    while board[[(csr_pos.1 - 1) as usize, (csr_pos.0 - 1) as usize]].content
                        != 0
                    {
                        board.randomize();
                        board.set_nums();
                    }
                    generated = true;
                }
                if board.open((csr_pos.1 - 1) as usize, (csr_pos.0 - 1) as usize)
                {
                    board.reveal();
                    render!();
                    write!(stdout, "\n\r")?;
                    stdout.flush()?;
                    break;
                }
            }
            Key::Char('f') =>
            {
                board.toggle_flag((csr_pos.1 - 1) as usize, (csr_pos.0 - 1) as usize)
            }
            _ =>
            {}
        }

        if board.spaces_left == board.mines()
        {
            board.flag_all();
        }
        render!();
        if board.spaces_left == board.mines()
        {
            write!(stdout, "\n\r")?;
            stdout.flush()?;
            break;
        }
        write!(
            stdout,
            "{}",
            termion::cursor::Goto(csr_pos.0 * 4 - 1, csr_pos.1 * 2)
        )?;
        stdout.flush()?;
    }

    write!(stdout, "{}", termion::cursor::Show)?;
    stdout.flush()?;
    Ok(())
}
