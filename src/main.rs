extern crate termion;
extern crate clap;

use clap::{App, Arg};
use std::io::{self, stdin, stdout, Write};
use std::path;
use std::ffi::OsStr;
use termion::clear;
use termion::cursor;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use std::fs;
use std::cmp::min;

struct Cursor {
	row: usize,
	column: usize,
}

struct Kiro {
	buffer: Vec<Vec<char>>,
	cursor: Cursor,
	path: Option<path::PathBuf>,
}

impl Default for Kiro {
	fn default() -> Self {
		Self {
			buffer: vec![Vec::new()],
			cursor: Cursor { row: 0, column: 0 },
			path: None,
		}
	}
}

impl Kiro {
	// ファイルを読み込む
	fn open(&mut self, path: &path::Path) {
		self.buffer = fs::read_to_string(path)
			.ok()
			.map(|s| {
				let buffer: Vec<Vec<char>> = s
					.lines()
					.map(|line| line.trim_end().chars().collect())
					.collect();
				if buffer.is_empty() {
					vec![Vec::new()]
				} else {
					buffer
				}
			})
			.unwrap_or_else(|| vec![Vec::new()]);
		self.path = Some(path.into());
		self.cursor = Cursor {row: 0, column: 0};
		//self.row_offset = 0;
	}
	// 画面描写
	fn draw<T: Write>(&self, out: &mut T) {
		write!(out, "{}", clear::All);
		write!(out, "{}", cursor::Goto(1, 1));

		for line in &self.buffer {
			for &c in line {
				write!(out, "{}", c);
			}
			write!(out, "\r\n");
		}

		write!(
			out,
			"{}",
			cursor::Goto(self.cursor.column as u16 + 1, self.cursor.row as u16 + 1)
		);
		out.flush().unwrap();
	}
	// カーソルUP
	fn cursor_up(&mut self) {
		if self.cursor.row > 0 {
			self.cursor.row -= 1;
			self.cursor.column = min(self.buffer[self.cursor.row].len(), self.cursor.column);
		}
	}
	// カーソルDOWN
	fn cursor_down(&mut self) {
		if self.cursor.row + 1 < self.buffer.len() {
			self.cursor.row += 1;
			self.cursor.column = min(self.cursor.column, self.buffer[self.cursor.row].len());
		}
	}
	// カーソルLEFT
	fn cursor_left(&mut self) {
		if self.cursor.column > 1 {
			self.cursor.column -= 1;
		}
	}
	// カーソルRIGHT
	fn cursor_right(&mut self) {
		self.cursor.column = min(self.cursor.column + 1, self.buffer[self.cursor.row].len());
	}
	// 文字入力
	fn insert(&mut self, c: char) {
		if c == '\n' {
			let rest: Vec<char> = self.buffer[self.cursor.row].drain(self.cursor.column..).collect();
			self.buffer.insert(self.cursor.row + 1, rest);
			self.cursor.row += 1;
			self.cursor.column = 0;
		} else if !c.is_control() {
			self.buffer[self.cursor.row].insert(self.cursor.column, c);
			self.cursor_right();
		}
	}
	// 文字消去
	fn delete(&mut self) {
		if self.cursor.column > 0 {
			let mut later = self.buffer[self.cursor.row].split_off(self.cursor.column);
			self.buffer[self.cursor.row].pop();
			self.buffer[self.cursor.row].append(&mut later);
			self.cursor.column -= 1;
		}
	}
	// 保存
	fn save(&self) {
		if let Some(path) = self.path.as_ref() {
			if let Ok(mut file) = fs::File::create(path) {
				for line in &self.buffer {
					for &c in line {
						write!(file, "{}", c).unwrap();
					}
					writeln!(file).unwrap();
				}
			}
		}
	}
}

fn main() {
	let matches = App::new("text_editer")
		.about("A text editor")
		.bin_name("text_editer")
		.arg(Arg::with_name("file").required(true))
		.get_matches();

	let file_path: &OsStr = matches.value_of_os("file").unwrap();

	let mut state = Kiro::default();

	state.open(path::Path::new(file_path));

	let stdin = stdin();

	let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());

	state.draw(&mut stdout);

	for evt in stdin.events() {


		match evt.unwrap() {
			Event::Key(Key::Char(c)) => {
				state.insert(c);
			},
			Event::Key(Key::Backspace) => {
				state.delete();
			},
			Event::Key(Key::Up) => {
				state.cursor_up();
			},
			Event::Key(Key::Down) => {
				state.cursor_down();
			},
			Event::Key(Key::Left) => {
				state.cursor_left();
			},
			Event::Key(Key::Right) => {
				state.cursor_right();
			},
			Event::Key(Key::Ctrl('s')) => {
				state.save();
			}
			Event::Key(Key::Ctrl('c')) => {
				return;
			},
			_ => {
			}
		}
		state.draw(&mut stdout);
	}
}
