use std::io::{self, stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

use crate::catalog::Catalog;
use crate::decide::{self, GoldenRun};
use crate::model;
use crate::types::{EngineResult, RefereeKind, Snap};

struct App {
    input: String,
    snap_idx: usize,
    live_mode: bool,
    live_snap: Option<Snap>,
    live_error: Option<String>,
    referee: RefereeKind,
    result: Option<EngineResult>,
    status: String,
    tree_scroll: u16,
    golden_idx: usize,
    test_report: Option<String>,
}

impl App {
    fn new() -> Self {
        Self {
            input: String::new(),
            snap_idx: 0,
            live_mode: false,
            live_snap: None,
            live_error: None,
            referee: RefereeKind::Lexical,
            result: None,
            status: "type a phrase · enter to take".into(),
            tree_scroll: 0,
            golden_idx: 0,
            test_report: None,
        }
    }
}

pub fn run(cat: &Catalog) -> Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(ratatui::backend::CrosstermBackend::new(out))?;
    let mut app = App::new();
    let result = event_loop(&mut terminal, &mut app, cat);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn event_loop(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    app: &mut App,
    cat: &Catalog,
) -> Result<()> {
    loop {
        terminal.draw(|f| draw(f, app, cat))?;
        if !event::poll(Duration::from_millis(200))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if handle(app, cat, key, terminal)? {
            break;
        }
    }
    Ok(())
}

fn handle(
    app: &mut App,
    cat: &Catalog,
    key: KeyEvent,
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
) -> Result<bool> {
    if key.kind != event::KeyEventKind::Press && key.kind != event::KeyEventKind::Repeat {
        // Some terminals repeat; treat Press. Crossterm on Linux may fire Release too.
        if key.kind == event::KeyEventKind::Release {
            return Ok(false);
        }
    }
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match (key.code, ctrl) {
        (KeyCode::Char('c'), true) | (KeyCode::Char('q'), true) => return Ok(true),
        (KeyCode::Esc, _) => {
            if app.input.is_empty() {
                return Ok(true);
            }
            app.input.clear();
        }
        (KeyCode::Char('q'), false) if app.input.is_empty() => return Ok(true),
        (KeyCode::Char('s'), true) => {
            app.live_mode = false;
            app.snap_idx = (app.snap_idx + 1) % cat.snaps.len().max(1);
            app.status = format!("snap {}", current_snap(app, cat).id);
        }
        (KeyCode::Char('r'), true) => {
            app.referee = app.referee.next();
            app.status = format!("referee {}", app.referee);
        }
        (KeyCode::Char('l'), true) => match crate::host::live_snap() {
            Ok(s) => {
                app.live_snap = Some(s);
                app.live_mode = true;
                app.live_error = None;
                app.status = "live host snap".into();
            }
            Err(e) => {
                app.live_error = Some(e.to_string());
                app.live_mode = false;
                app.status = format!("live snap failed: {e}");
            }
        },
        (KeyCode::Char('g'), true) => {
            if let Some(g) = cat.goldens.get(app.golden_idx) {
                app.input = g.utterance.clone();
                if let Some(i) = cat.snaps.iter().position(|s| s.id == g.snap) {
                    app.snap_idx = i;
                    app.live_mode = false;
                }
                app.status = format!("golden {} (expect {:?})", g.id, g.expect_page);
                app.golden_idx = (app.golden_idx + 1) % cat.goldens.len().max(1);
            }
        }
        (KeyCode::Char('t'), true) => {
            app.status = "running lexical goldens…".into();
            terminal.draw(|f| draw(f, app, cat))?;
            app.test_report = Some(run_report(cat));
            app.status = "lexical goldens finished".into();
        }
        (KeyCode::Char('p'), true) => {
            if let Some(p) = cat.phrases.get(app.golden_idx % cat.phrases.len().max(1)) {
                app.input = p.utterance.clone();
                app.status = format!("phrase {} (expect {:?})", p.id, p.expect_page);
                app.golden_idx += 1;
            }
        }
        (KeyCode::Enter, _) => {
            evaluate(app, cat, terminal)?;
        }
        (KeyCode::Backspace, _) => {
            app.input.pop();
        }
        (KeyCode::Up, _) => {
            app.tree_scroll = app.tree_scroll.saturating_sub(1);
        }
        (KeyCode::Down, _) => {
            app.tree_scroll = app.tree_scroll.saturating_add(1);
        }
        (KeyCode::Char(c), false) if !c.is_control() => {
            app.input.push(c);
        }
        _ => {}
    }
    Ok(false)
}

fn evaluate(
    app: &mut App,
    cat: &Catalog,
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
) -> Result<()> {
    let utterance = app.input.trim().to_string();
    if utterance.is_empty() {
        app.status = "type something first".into();
        return Ok(());
    }
    let snap = current_snap(app, cat).clone();
    if app.referee != RefereeKind::Lexical {
        if !model::available(app.referee) {
            app.status = format!("{} is not available on this box", app.referee);
            return Ok(());
        }
        app.status = format!("calling {}…", app.referee);
        terminal.draw(|f| draw(f, app, cat))?;
    }
    match crate::eval_on(cat, &utterance, &snap, app.referee) {
        Ok(r) => {
            let summary = match &r.take.page_id {
                Some(id) => format!("take {id}  conf={:.2}", r.take.confidence),
                None => format!("silence — {}", r.take.reason),
            };
            app.status = summary;
            app.tree_scroll = 0;
            app.result = Some(r);
            app.test_report = None;
        }
        Err(e) => {
            app.status = format!("eval failed: {e}");
        }
    }
    Ok(())
}

fn current_snap<'a>(app: &'a App, cat: &'a Catalog) -> &'a Snap {
    if app.live_mode {
        if let Some(s) = app.live_snap.as_ref() {
            return s;
        }
    }
    cat.snaps.get(app.snap_idx).unwrap_or(&cat.snaps[0])
}

fn run_report(cat: &Catalog) -> String {
    let mut lines = Vec::new();
    let mut pass = 0;
    let mut fail = 0;
    for g in &cat.goldens {
        let r: GoldenRun = decide::run_golden(cat, g);
        if r.pass {
            pass += 1;
            lines.push(format!("  ✓ {}", r.id));
        } else {
            fail += 1;
            lines.push(format!(
                "  ✗ {}  got {:?} expected {:?} slots_ok={}",
                r.id, r.got, r.expected, r.slots_ok
            ));
        }
    }
    lines.insert(
        0,
        format!(
            "lexical goldens  {pass} pass / {fail} fail / {} total",
            pass + fail
        ),
    );
    lines.join("\n")
}

fn draw(f: &mut Frame, app: &App, cat: &Catalog) {
    let snap = current_snap(app, cat);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(f.area());

    let laya = if model::available(RefereeKind::Laya) {
        "laya ✓"
    } else {
        "laya ✗  (edgejev serve :8009)"
    };
    let snap_label = if app.live_mode {
        format!("LIVE:{}", snap.id)
    } else {
        snap.id.clone()
    };
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " vikett ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  snap "),
        Span::styled(
            snap_label,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("  {}  guest={}  ", snap.place, snap.guest)),
        Span::styled(
            format!("{}", app.referee),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("  {laya}")),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" rig "));
    f.render_widget(header, chunks[0]);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title(" utterance "));
    f.render_widget(input, chunks[1]);
    f.set_cursor_position(ratatui::layout::Position::new(
        chunks[1].x + 1 + app.input.chars().count() as u16,
        chunks[1].y + 1,
    ));

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(chunks[2]);

    if let Some(report) = &app.test_report {
        let p = Paragraph::new(report.as_str())
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title(" goldens "));
        f.render_widget(p, body[0]);
        f.render_widget(outcome_widget(app, snap), body[1]);
    } else {
        let tree = app
            .result
            .as_ref()
            .map(|r| r.trace.render())
            .unwrap_or_else(|| "decision tree appears here after enter".into());
        let p = Paragraph::new(tree)
            .wrap(Wrap { trim: false })
            .scroll((app.tree_scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" decision tree  ↑↓ scroll "),
            );
        f.render_widget(p, body[0]);
        f.render_widget(outcome_widget(app, snap), body[1]);
    }

    let help = Paragraph::new(
        "enter eval   C-s snap   C-r referee   C-l live host   C-g next golden   C-p phrase   C-t run goldens   q/esc quit",
    )
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default().borders(Borders::ALL).title(format!(" {} ", app.status)));
    f.render_widget(help, chunks[3]);
}

fn outcome_widget<'a>(app: &'a App, snap: &'a Snap) -> Paragraph<'a> {
    let text = match &app.result {
        None => format!(
            "snap {}\n{}\nwho {}  occupants {:?}\nclients {}\nallowlist {:?}\n\nexpected walk / ask will land here.",
            snap.id,
            snap.title,
            snap.who,
            snap.occupants,
            snap.clients.len(),
            snap.allowlist,
        ),
        Some(r) => {
            let mut s = String::new();
            s.push_str(&format!("take  {}\n", r.take.page_id.as_deref().unwrap_or("∅ silence")));
            s.push_str(&format!("conf  {:.2}\n", r.take.confidence));
            s.push_str(&format!("why   {}\n", r.take.reason));
            if !r.take.slots.is_empty() {
                s.push_str("slots\n");
                for (k, v) in &r.take.slots {
                    s.push_str(&format!("  {k} = {v}\n"));
                }
            }
            if r.confirm {
                s.push_str("policy  CONFIRM — walk waits on a yes\n");
            }
            if let Some(w) = &r.walk {
                s.push_str(&format!("\nexpected walk\n  {}\n  driver {}\n", w.command, w.driver));
            }
            if let Some(a) = &r.answer {
                s.push_str(&format!("\nask json\n  {a}\n"));
            }
            if !r.remaining.is_empty() {
                s.push_str(&format!("\ncompound rest  {:?}\n", r.remaining));
            }
            s.push_str(&format!("\nlive {}   dead {}\n", r.live.len(), r.dead.len()));
            s
        }
    };
    Paragraph::new(text).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" expected outcome "),
    )
}

#[allow(dead_code)]
fn dim(area: Rect) -> Rect {
    area
}

#[allow(dead_code)]
fn popup(f: &mut Frame, area: Rect, text: &str) {
    let block = Block::default().borders(Borders::ALL).title(" note ");
    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(text).block(block), area);
}
