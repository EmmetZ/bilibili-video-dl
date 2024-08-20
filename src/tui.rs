use ratatui::{
    buffer::Buffer,
    crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget,
    },
    Terminal,
};
use std::io::{self, stdout, Stdout};

use crate::http::download::Task;

type Tui = Terminal<CrosstermBackend<Stdout>>;

pub struct SelectionUI<'u> {
    should_exit: bool,
    video_list: VideoList<'u>,
}

struct VideoList<'l> {
    videos: &'l [Task],
    state: ListState,
    is_selected: Vec<bool>,
    select_all: bool,
}

impl<'u> SelectionUI<'u> {
    pub fn new(video_list: &'u [Task]) -> Self {
        let vl = VideoList {
            videos: video_list,
            state: ListState::default(),
            is_selected: vec![true; video_list.len()],
            select_all: true,
        };
        Self {
            should_exit: false,
            video_list: vl,
        }
    }
}

impl SelectionUI<'_> {
    pub fn run(&mut self) -> io::Result<()> {
        let mut terminal = init_terminal()?;
        while !self.should_exit {
            terminal.draw(|f| f.render_widget(&mut *self, f.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }
        restore()?;
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Up => {
                self.select_prev();
            }
            KeyCode::Down => {
                self.select_next();
            }
            KeyCode::Char('d') => {
                self.toggle_status();
            }
            KeyCode::Enter => self.should_exit = true,

            KeyCode::Char('n') => {
                println!("取消下载");
                self.toggle_all_status(Some(false));
                self.should_exit = true;
            }
            KeyCode::Char('a') => {
                self.toggle_all_status(None);
            }
            _ => {}
        }
    }

    fn select_next(&mut self) {
        self.video_list.state.select_next();
    }

    fn select_prev(&mut self) {
        self.video_list.state.select_previous();
    }

    fn toggle_status(&mut self) {
        if let Some(i) = self.video_list.selected() {
            self.video_list.is_selected[i] = !self.video_list.is_selected[i];
        }
    }

    fn toggle_all_status(&mut self, default: Option<bool>) {
        if let Some(s) = default {
            self.video_list.is_selected = vec![s; self.video_list.len()];
        } else {
            self.video_list.select_all = !self.video_list.select_all;
            self.video_list.is_selected = vec![self.video_list.select_all; self.video_list.len()];
        }
    }

    pub fn get_selection(self) -> Vec<bool> {
        self.video_list.is_selected
    }
}

impl VideoList<'_> {
    fn len(&self) -> usize {
        self.videos.len()
    }

    fn selected(&self) -> Option<usize> {
        self.state.selected()
    }
}

impl Widget for &mut SelectionUI<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [main_area, foot_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(area);

        SelectionUI::render_footer(foot_area, buf);
        self.render_list(main_area, buf);
    }
}

impl SelectionUI<'_> {
    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new(
            "<↑/↓>: 上下移动; <d>: 选择 / 取消选择; <a>: 全选 / 全不选; <Enter>: 确认; <n>: 取消下载",
        )
        .centered()
        .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new().title("Select Video").borders(Borders::ALL);

        let items: Vec<ListItem> = self
            .video_list
            .videos
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let is_checked = if self.video_list.is_selected[i] {
                    "[✓]"
                } else {
                    "[ ]"
                };
                let content = format!("{} {}", is_checked, v.title);
                let style = if self.video_list.is_selected[i] {
                    Style::default().fg(Color::Rgb(255, 167, 38))
                } else {
                    Style::default().fg(Color::Rgb(120, 144, 156))
                };
                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol(">> ")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.video_list.state);
    }
}

fn init_terminal() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn wait() {
    println!("点击任意键继续...");
    loop {
        if let Event::Key(k) = event::read().expect("Failed to read input") {
            if k.kind == KeyEventKind::Press {
                break;
            }
        }
    }
}

pub fn select_download_video(video_list: Vec<Task>, flag: Vec<bool>) -> Vec<Task> {
    video_list
        .into_iter()
        .zip(flag)
        .filter(|(_, selected)| *selected)
        .map(|(video, _)| video)
        .collect()
}
