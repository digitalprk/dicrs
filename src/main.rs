use std::cmp::{max, min};
use std::fs;
use std::io::{self, stdin, Write};
use termion::event::{Event, Key, MouseButton, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, SelectableList, Text, Widget};
use tui::Terminal;
use unicode_width::UnicodeWidthStr;

use termion::cursor::Goto;

extern crate clipboard;
extern crate rusqlite;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use rusqlite::Connection;

static DICPATH: &str = "dics/";
static DICEXTENSION: &str = ".db";
static APPTERMTITLE: &str = "\x1b]0;dic.rs\x07";

struct DicEntry {
    index: usize,
    word: String,
    definition: String,
}

#[derive(Copy, Clone)]
struct Rect {
    x: u16,
    y: u16,
    height: u16,
    width: u16,
}

impl Rect {
    fn contains(self, x: u16, y: u16) -> bool {
        (x <= self.x + self.width) & (x >= self.x) & (y >= self.y) & (y <= self.y + self.height)
    }
    fn new(x: u16, y: u16, height: u16, width: u16) -> Rect {
        Rect {
            x,
            y,
            height,
            width,
        }
    }
    fn default() -> Rect {
        Rect {
            x: u16::default(),
            y: u16::default(),
            height: u16::default(),
            width: u16::default(),
        }
    }
}

struct App {
    input: String,
    definition: String,
    selected_index: usize,
    dictionary_index: usize,
    database_path: String,
    conn: Connection,
    word_index: Vec<String>,
}

impl Default for DicEntry {
    fn default() -> DicEntry {
        DicEntry {
            index: usize::default(),
            word: String::new(),
            definition: String::new(),
        }
    }
}

impl App {
    fn default() -> App {
        App {
            input: String::new(),
            definition: String::new(),
            selected_index: usize::default(),
            dictionary_index: usize::default(),
            database_path: String::new(),
            conn: Connection::open_in_memory().unwrap(),
            word_index: Vec::new(),
        }
    }

    fn create(&mut self, db_path: String) {
        self.selected_index = 0;
        self.database_path = db_path.clone();
        self.conn = Connection::open(&db_path).unwrap();
        self.word_index = retrieve_db_index(&self.conn);
    }

    fn update_by_index(&mut self, i: i32) {
        let mut new_index: i32 = max(0, self.selected_index as i32 + i);
        new_index = min(new_index, self.word_index.len() as i32);
        self.selected_index = new_index as usize;
        let new_word = self.word_index[self.selected_index].clone();
        self.definition = self.query_db(&new_word).definition;
    }

    fn update(&mut self) {
        let new_word = self.word_index[self.selected_index].clone();
        self.definition = self.query_db(&new_word).definition;
    }

    fn query_db(&mut self, query: &str) -> DicEntry {
        let mut stmt = self
            .conn
            .prepare("SELECT ROWID, word, definition FROM dictionary WHERE word LIKE :query")
            .unwrap();
        let mut res = DicEntry::default();
        let wild_card_query = [query, "%"].concat();
        let mut rows = stmt.query_named(&[(":query", &wild_card_query)]).unwrap();
        if let Ok(Some(row)) = rows.next() {
            let rowid: u32 = row.get(0).unwrap();
            res.index = (rowid - 1) as usize;
            res.word = row.get(1).unwrap();
            let def: String = row.get(2).unwrap();
            res.definition = def.replace("\r", "\n");
        } else {
            res.definition = "Not found!".to_string();
        }
        res
    }
}

fn list_databases() -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    for entry in fs::read_dir(DICPATH).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let filename = path.file_name().ok_or("No filename").unwrap().to_str();
        let filename = filename.unwrap().to_string().replace(DICEXTENSION, "");
        res.push(filename);
    }
    res
}

fn retrieve_db_index(conn: &Connection) -> Vec<String> {
    let mut stmt = conn.prepare("SELECT word FROM dictionary").unwrap();
    let mut rows = stmt.query(rusqlite::NO_PARAMS).unwrap();
    let mut index = Vec::new();
    while let Ok(Some(row)) = rows.next() {
        index.push(row.get(0).unwrap());
    }
    index
}

fn main() -> Result<(), failure::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let databases = list_databases();
    let mut app = App::default();
    app.create([DICPATH, databases.get(0).unwrap(), DICEXTENSION].concat());
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    let mut index_list_rect = Rect::default();

    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Length(5),
                        Constraint::Min(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            let help_message =
                "Press Ctrl+C to leave, Ctrl+Y to copy, Left/Right to change dictionaries";

            Paragraph::new([Text::raw(&app.input)].iter())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Word"))
                .render(&mut f, chunks[1]);

            Paragraph::new([Text::raw(help_message)].iter()).render(&mut f, chunks[0]);
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Dictionaries"))
                .items(&databases)
                .select(Some(app.dictionary_index))
                .highlight_style(
                    Style::default()
                        .fg(Color::Blue)
                        .bg(Color::White)
                        .modifier(Modifier::BOLD),
                )
                .render(&mut f, chunks[2]);

            {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                    .split(chunks[3]);

                SelectableList::default()
                    .block(Block::default().borders(Borders::ALL).title("Index"))
                    .items(&app.word_index)
                    .highlight_style(
                        Style::default()
                            .fg(Color::Blue)
                            .bg(Color::White)
                            .modifier(Modifier::BOLD),
                    )
                    .select(Some(app.selected_index))
                    .render(&mut f, chunks[0]);
                index_list_rect = Rect::new(
                    chunks[0].x,
                    chunks[0].y + 2,
                    chunks[0].height - 2,
                    chunks[0].width,
                );

                Paragraph::new([Text::raw(&app.definition)].iter())
                    .wrap(true)
                    .block(Block::default().borders(Borders::ALL).title("Definition"))
                    .render(&mut f, chunks[1]);
            }
        })?;
        write!(terminal.backend_mut(), "{}", APPTERMTITLE)?;
        write!(
            terminal.backend_mut(),
            "{}",
            Goto(4 + app.input.width() as u16, 5)
        )?;
        io::stdout().flush().ok();

        let mut events = stdin().events();
        match events.next().unwrap().unwrap() {
            Event::Key(Key::Char('\n')) => {
                let query_term: String = app.input.drain(..).collect();
                let entry = app.query_db(&query_term);
                app.definition = entry.definition;
                app.selected_index = entry.index;
            }
            Event::Key(Key::Char(c)) => {
                app.input.push(c);
            }
            Event::Key(Key::Down) => {
                app.update_by_index(1);
            }
            Event::Key(Key::Up) => {
                app.update_by_index(-1);
            }
            Event::Key(Key::PageDown) => {
                app.update_by_index(10);
            }
            Event::Key(Key::PageUp) => {
                app.update_by_index(-10);
            }
            Event::Key(Key::Ctrl('c')) => {
                break;
            }
            Event::Key(Key::Ctrl('y')) => {
                ctx.set_contents(app.definition.to_owned()).unwrap();
            }
            Event::Key(Key::Backspace) => {
                app.input.pop();
            }
            Event::Key(Key::Right) => {
                app.dictionary_index = (app.dictionary_index + 1) % databases.len();
                app.create(
                    [
                        DICPATH,
                        databases.get(app.dictionary_index).unwrap(),
                        DICEXTENSION,
                    ]
                    .concat(),
                );
            }
            Event::Key(Key::Left) => {
                app.dictionary_index = (app.dictionary_index - 1) % databases.len();
                app.create(
                    [
                        DICPATH,
                        databases.get(app.dictionary_index).unwrap(),
                        DICEXTENSION,
                    ]
                    .concat(),
                );
            }
            Event::Mouse(me) => match me {
                MouseEvent::Press(MouseButton::WheelDown, _, _) => {
                    app.update_by_index(1);
                }
                MouseEvent::Press(MouseButton::WheelUp, _, _) => {
                    app.update_by_index(-1);
                }
                MouseEvent::Press(MouseButton::Left, x, y) => {
                    if index_list_rect.contains(x, y) {
                        let number_of_elements = (index_list_rect.height) as usize;
                        let index_clicked = (y - index_list_rect.y) as usize;
                        if app.selected_index >= number_of_elements {
                            app.selected_index -= number_of_elements - index_clicked - 1;
                        } else {
                            app.selected_index = index_clicked;
                        }
                        app.update();
                    }
                }
                _ => {}
            },

            _ => {}
        }
    }
    Ok(())
}
