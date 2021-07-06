use arrayvec::ArrayVec;
use rand::Rng;
use std::ops::{Index, IndexMut};

const UL: &str = "\u{250F}";
const UR: &str = "\u{2513}";
const DL: &str = "\u{2517}";
const DR: &str = "\u{251B}";
const LT: &str = "\u{2523}";
const RT: &str = "\u{252B}";
const UT: &str = "\u{2533}";
const DT: &str = "\u{253B}";
const HL: &str = "\u{2501}";
const VL: &str = "\u{2503}";
const QD: &str = "\u{254B}";

const MINE: &str = "\x1b[91;1m\u{273B}\x1b[0m";
const CELL: &str = "\u{25A0}";
const FLAG: &str = "\x1b[33m\u{2691}\x1b[0m";

pub struct Cell
{
    content: u8,
    opened: bool,
    flagged: bool,
}

impl Cell
{
    pub fn content(&self) -> u8
    {
        self.content
    }
}

impl Cell
{
    fn new(content: u8) -> Self
    {
        Self {
            content,
            opened: false,
            flagged: false,
        }
    }
}

impl Default for Cell
{
    fn default() -> Self
    {
        Self::new(0)
    }
}

impl std::fmt::Debug for Cell
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.content)
    }
}

pub struct Board<const ROWS: usize, const COLUMNS: usize, const MINES: usize>
{
    board: ArrayVec<ArrayVec<Cell, COLUMNS>, ROWS>,
    flags_left: usize,
    spaces_left: usize,
}

impl<const ROWS: usize, const COLUMNS: usize, const MINES: usize> Board<ROWS, COLUMNS, MINES>
{
    pub fn clear(&mut self)
    {
        for r in 0..ROWS
        {
            for c in 0..COLUMNS
            {
                self[(r, c)] = Cell::default();
            }
        }
        self.refresh_vals();
    }

    pub fn randomize(&mut self)
    {
        self.clear();
        let mut mines = MINES.min(ROWS * COLUMNS);
        let mut rng = rand::thread_rng();
        while mines > 0
        {
            let pos = (rng.gen_range(0..ROWS), rng.gen_range(0..COLUMNS));
            if self[pos].content != 9
            {
                self[pos].content = 9;
                mines -= 1;
            }
        }
        self.refresh_vals();
    }

    pub fn set_nums(&mut self)
    {
        for r in 0..ROWS
        {
            for c in 0..COLUMNS
            {
                if self[(r, c)].content == 9
                {
                    continue;
                }

                self[(r, c)].content = 0;

                let adjs = Self::adjs(r, c);

                for adj in adjs
                {
                    if self[adj].content == 9
                    {
                        self[(r, c)].content += 1;
                    }
                }
            }
        }
    }

    pub fn draw(&self, buf: &mut dyn std::io::Write) -> std::io::Result<()>
    {
        macro_rules! n {
            () => (1);
            ($n:expr) => ($n);
        }
        macro_rules! push {
            ($($arg:expr $(; $count:expr)?),*) => {
                $(
                    for _ in 0..n!($($count)?)
                    {
                        write!(buf, "{}", $arg)?;
                    }
                )*
            };
        }
        macro_rules! pop {
            () => {
                write!(buf, "\x08")?;
            };
            ($count:expr) => {
                for _ in 0..$count
                {
                    write!(buf, "\x08")?;
                }
            };
        }
        push!("\x1b[90m", UL);
        for _ in 0..COLUMNS
        {
            push!(HL;3, UT);
        }
        pop!();
        push!(UR, "\x1b[0m", "\n\r");
        for r in 0..ROWS
        {
            for c in 0..COLUMNS
            {
                let repr = if self[(r, c)].opened
                {
                    match self[(r, c)].content
                    {
                        0 => " ",
                        1 => "\x1b[94m1\x1b[0m",
                        2 => "\x1b[32m2\x1b[0m",
                        3 => "\x1b[91m3\x1b[0m",
                        4 => "\x1b[35m4\x1b[0m",
                        5 => "\x1b[33m5\x1b[0m",
                        6 => "\x1b[96m6\x1b[0m",
                        7 => "\x1b[30m7\x1b[0m",
                        8 => "\x1b[90m8\x1b[0m",
                        9 => MINE,
                        _ => " ",
                    }
                }
                else if self[(r, c)].flagged
                {
                    FLAG
                }
                else
                {
                    CELL
                };
                push!("\x1b[90m", VL, "\x1b[0m", " ", repr, "\x1b[90m", " ", "\x1b[0m");
            }
            push!("\x1b[90m", VL, "\n\r", LT);
            for _ in 0..COLUMNS
            {
                push!(HL;3, QD);
            }
            pop!();
            push!(RT, "\x1b[0m\n\r");
        }
        push!("\x1b[1A\x1b[90m\r");
        push!(DL);
        for _ in 0..COLUMNS
        {
            push!(HL;3, DT);
        }
        pop!();
        push!(DR, "\x1b[0m");

        Ok(())
    }

    pub fn open(&mut self, r: usize, c: usize) -> bool
    {
        if self[(r, c)].flagged
        {
            return false;
        }
        if !self[(r, c)].opened
        {
            self[(r, c)].opened = true;
            self.spaces_left -= 1;
        }
        if self[(r, c)].content == 9
        {
            return true;
        }
        let mut flags = 0;
        let adjs = Self::adjs(r, c);
        for &adj in adjs.iter()
        {
            if self[adj].flagged
            {
                flags += 1;
            }
        }
        if self[(r, c)].content == flags
        {
            for &adj in adjs.iter()
            {
                if !(self[adj].flagged || self[adj].opened)
                {
                    if self.open(adj.0, adj.1)
                    {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    pub fn toggle_flag(&mut self, r: usize, c: usize)
    {
        if !self[(r, c)].opened
        {
            if self[(r, c)].flagged
            {
                self[(r, c)].flagged = false;
                self.flags_left += 1;
            }
            else if self.flags_left > 0
            {
                self[(r, c)].flagged = true;
                self.flags_left -= 1;
            }
        }
        else
        {
            let mut closed = 0;
            let adjs = Self::adjs(r, c);
            for &adj in adjs.iter()
            {
                if !self[adj].opened
                {
                    closed += 1;
                }
            }
            if closed == self[(r, c)].content
            {
                for &adj in adjs.iter()
                {
                    if !(self[adj].opened || self[adj].flagged)
                    {
                        self.toggle_flag(adj.0, adj.1);
                    }
                }
            }
        }
    }

    pub fn reveal(&mut self)
    {
        for r in 0..ROWS
        {
            for c in 0..COLUMNS
            {
                self[(r, c)].opened = true;
            }
        }
        self.refresh_vals();
    }

    pub fn flag_all(&mut self)
    {
        for r in 0..ROWS
        {
            for c in 0..COLUMNS
            {
                if !self[(r, c)].opened
                {
                    self[(r, c)].flagged = true;
                }
            }
        }
        self.refresh_vals();
    }

    pub fn rows(&self) -> usize
    {
        ROWS
    }
    pub fn columns(&self) -> usize
    {
        COLUMNS
    }
    pub fn mines(&self) -> usize
    {
        MINES
    }
    pub fn flags_left(&self) -> usize
    {
        self.flags_left
    }
    pub fn spaces_left(&self) -> usize
    {
        self.spaces_left
    }
    
    pub fn blank() -> Self
    {
        let mut new = Self {
            board: ArrayVec::new(),
            flags_left: MINES,
            spaces_left: ROWS * COLUMNS,
        };

        for r in 0..ROWS
        {
            new.board.push(ArrayVec::new());
            for _ in 0..COLUMNS
            {
                new.board[r].push(Cell::default());
            }
        }

        new
    }
    
    pub fn adjs(r: usize, c: usize) -> Vec<(usize, usize)>
    {
        let mut adjs = Vec::<(usize, usize)>::new();
        let is_valid = |pos: &(usize, usize)| pos != &(r, c) && pos.0 < ROWS && pos.1 < COLUMNS;
        let lbndr = if r == 0 { r } else { r - 1 };
        let lbndc = if c == 0 { c } else { c - 1 };
        for adjr in lbndr..=(r + 1)
        {
            for adjc in lbndc..=(c + 1)
            {
                let pos = (adjr, adjc);
                if is_valid(&pos)
                {
                    adjs.push(pos);
                }
            }
        }
        adjs
    }

    fn refresh_vals(&mut self)
    {
        self.flags_left = MINES;
        self.spaces_left = ROWS * COLUMNS;
        for r in 0..ROWS
        {
            for c in 0..COLUMNS
            {
                if self[(r, c)].flagged
                {
                    self.flags_left -= 1;
                }
                if self[(r, c)].opened
                {
                    self.spaces_left -= 1;
                }
            }
        }
    }
}

impl<const ROWS: usize, const COLUMNS: usize, const MINES: usize> Default
    for Board<ROWS, COLUMNS, MINES>
{
    fn default() -> Self
    {
        Self::blank()
    }
}

impl<const ROWS: usize, const COLUMNS: usize, const MINES: usize> Index<(usize, usize)>
    for Board<ROWS, COLUMNS, MINES>
{
    type Output = Cell;

    fn index(&self, idx: (usize, usize)) -> &Self::Output
    {
        &self.board[idx.0][idx.1]
    }
}

impl<const ROWS: usize, const COLUMNS: usize, const MINES: usize> IndexMut<(usize, usize)>
    for Board<ROWS, COLUMNS, MINES>
{
    fn index_mut(&mut self, idx: (usize, usize)) -> &mut Self::Output
    {
        &mut self.board[idx.0][idx.1]
    }
}

pub mod difficulty
{
    use super::Board;

    pub type Beginner = Board<9, 9, 10>;
    pub type Medium = Board<16, 16, 40>;
    pub type Expert = Board<16, 30, 99>;
}
