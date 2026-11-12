use super::{
    Effect, Input, SEARCH_LIMIT, control, runner,
    terminal::{Frame, Line, Tone},
    text, view,
    viewport::Buffer,
};
use cirnetrules::operator::{
    CheckReport, Finding, Status,
    presentation::{self, GROUPS, Group, STATUS_ORDER},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{collections::BTreeSet, ops::Range};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Entry {
    Group(Group, usize),
    Finding(usize),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    List,
    Detail,
}

#[derive(Clone, Copy)]
enum FilterKind {
    Status,
    Repository,
}

struct FilterMenu {
    kind: FilterKind,
    selected: usize,
}

struct JsonView {
    text: Buffer,
    scroll: usize,
    limit: usize,
}

struct Hit {
    rows: Range<u16>,
    columns: Range<u16>,
    kind: HitKind,
}

enum HitKind {
    Entry(usize),
    Detail,
    Filter(usize),
}

type PanelLine = (Tone, String);

pub(super) struct CheckView {
    entries: Vec<Entry>,
    selected: usize,
    first: usize,
    page: usize,
    groups_open: [bool; 3],
    status: Option<Status>,
    repo: Option<String>,
    repos: Vec<String>,
    filter: Option<FilterMenu>,
    query: String,
    search: Option<Input>,
    focus: Focus,
    detail_scroll: usize,
    detail_limit: usize,
    split: bool,
    json: Option<JsonView>,
    hits: Vec<Hit>,
}

impl CheckView {
    pub(super) fn new(report: &CheckReport) -> Self {
        let mut view = Self {
            entries: Vec::new(),
            selected: 0,
            first: 0,
            page: 1,
            groups_open: [true, false, false],
            status: None,
            repo: None,
            repos: report
                .findings
                .iter()
                .filter_map(|row| row.repository.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            filter: None,
            query: String::new(),
            search: None,
            focus: Focus::List,
            detail_scroll: 0,
            detail_limit: 0,
            split: false,
            json: None,
            hits: Vec::new(),
        };
        view.refresh(report);
        if let Some(index) = view
            .entries
            .iter()
            .position(|entry| matches!(entry, Entry::Finding(_)))
        {
            view.selected = index;
        }
        view
    }

    pub(super) fn is_json(&self) -> bool {
        self.json.is_some()
    }

    pub(super) fn search_mut(&mut self) -> Option<&mut Input> {
        self.search.as_mut()
    }

    pub(super) fn searching(&self) -> bool {
        self.search.is_some()
    }

    pub(super) fn search_changed(&mut self, report: &CheckReport) {
        if let Some(search) = &self.search
            && self.query != search.value
        {
            self.query.clone_from(&search.value);
            self.refresh(report);
        }
    }

    fn finding<'a>(&self, report: &'a CheckReport) -> Option<&'a Finding> {
        if let Some(Entry::Finding(index)) = self.entries.get(self.selected) {
            return report.findings.get(*index);
        }
        None
    }

    pub(super) fn finding_text(&self, report: &CheckReport, vi: bool) -> Option<String> {
        self.finding(report)
            .map(|row| presentation::detail_lines(row, 88, vi).join("\n"))
    }

    pub(super) fn copy_value(&self, report: &CheckReport) -> Option<String> {
        let row = self.finding(report)?;
        let value = row
            .task_id
            .as_deref()
            .or(row.repository.as_deref())
            .or(row.binding_id.as_deref())
            .or(row.agent_id.as_deref())
            .unwrap_or(&row.code);
        Some(presentation::text(value))
    }

    fn refresh(&mut self, report: &CheckReport) {
        let previous = self.entries.get(self.selected).copied();
        let query = self.query.trim().to_lowercase();
        let filtered = self.status.is_some() || self.repo.is_some() || !query.is_empty();
        self.hits.clear();
        self.entries.clear();
        for group in GROUPS {
            let mut matching = Vec::new();
            for status in STATUS_ORDER
                .into_iter()
                .filter(|status| Group::of(*status) == group)
            {
                matching.extend(
                    report
                        .findings
                        .iter()
                        .enumerate()
                        .filter_map(|(index, row)| {
                            (row.status == status
                                && self.status.is_none_or(|expected| expected == row.status)
                                && self
                                    .repo
                                    .as_ref()
                                    .is_none_or(|repo| row.repository.as_ref() == Some(repo))
                                && (query.is_empty() || matches_query(row, &query)))
                            .then_some(index)
                        }),
                );
            }
            if !matching.is_empty() {
                self.entries.push(Entry::Group(group, matching.len()));
                if filtered || self.groups_open[group.index()] {
                    self.entries
                        .extend(matching.into_iter().map(Entry::Finding));
                }
            }
        }
        self.selected = previous
            .and_then(|entry| self.entries.iter().position(|row| same_entry(*row, entry)))
            .or_else(|| {
                self.entries
                    .iter()
                    .position(|entry| matches!(entry, Entry::Finding(_)))
            })
            .unwrap_or(0);
        self.first = 0;
        self.detail_scroll = 0;
    }

    fn select(&mut self, index: usize) {
        let selected = index.min(self.entries.len().saturating_sub(1));
        if selected != self.selected {
            self.selected = selected;
            self.detail_scroll = 0;
        }
    }

    pub(super) fn wheel(&mut self, _report: &CheckReport, lines: isize) -> bool {
        if let Some(json) = &mut self.json {
            return scroll(&mut json.scroll, json.limit, lines);
        }
        if let Some(filter) = &mut self.filter {
            let limit = filter_limit(filter.kind, &self.repos);
            return scroll(&mut filter.selected, limit, lines);
        }
        if self.focus == Focus::Detail {
            return scroll(&mut self.detail_scroll, self.detail_limit, lines);
        }
        let previous = self.selected;
        self.select(self.selected.saturating_add_signed(lines));
        previous != self.selected
    }

    pub(super) fn click(&mut self, report: &CheckReport, column: u16, row: u16) -> bool {
        let Some(hit) = self
            .hits
            .iter()
            .find(|hit| hit.rows.contains(&row) && hit.columns.contains(&column))
        else {
            return false;
        };
        match hit.kind {
            HitKind::Entry(index) => {
                self.focus = Focus::List;
                if self.selected == index
                    && matches!(self.entries.get(index), Some(Entry::Group(..)))
                {
                    self.open_selected(report);
                } else {
                    self.select(index);
                }
            }
            HitKind::Detail => self.focus = Focus::Detail,
            HitKind::Filter(index) => {
                if let Some(filter) = &mut self.filter {
                    filter.selected = index;
                }
                self.apply_filter(report);
            }
        }
        true
    }

    fn apply_filter(&mut self, report: &CheckReport) {
        let Some(filter) = self.filter.take() else {
            return;
        };
        match filter.kind {
            FilterKind::Status => {
                self.status = filter
                    .selected
                    .checked_sub(1)
                    .and_then(|index| STATUS_ORDER.get(index).copied())
            }
            FilterKind::Repository => {
                self.repo = filter
                    .selected
                    .checked_sub(1)
                    .and_then(|index| self.repos.get(index).cloned())
            }
        }
        self.focus = Focus::List;
        self.refresh(report);
    }

    fn open_selected(&mut self, report: &CheckReport) {
        match self.entries.get(self.selected).copied() {
            Some(Entry::Group(group, _)) => {
                // Explicit filters already show every matching item.
                if self.status.is_none() && self.repo.is_none() && self.query.trim().is_empty() {
                    self.groups_open[group.index()] ^= true;
                    self.refresh(report);
                }
            }
            Some(Entry::Finding(_)) => self.focus = Focus::Detail,
            None => {}
        }
    }

    pub(super) fn key(&mut self, report: &CheckReport, key: KeyEvent) -> Option<Effect> {
        let page = self.page.max(1) as isize;
        let delta = match key.code {
            KeyCode::Up => Some(-1),
            KeyCode::Down => Some(1),
            KeyCode::PageUp => Some(-page),
            KeyCode::PageDown => Some(page),
            KeyCode::Home => Some(isize::MIN),
            KeyCode::End => Some(isize::MAX),
            _ => None,
        };
        if self.json.is_some() {
            return Some(match key.code {
                KeyCode::Esc | KeyCode::Char('j') => {
                    self.json = None;
                    Effect::Redraw
                }
                _ if delta.is_some_and(|lines| self.wheel(report, lines)) => Effect::Redraw,
                _ => Effect::Idle,
            });
        }
        if let Some(search) = &mut self.search {
            return Some(match key.code {
                KeyCode::Enter | KeyCode::Esc => {
                    self.search = None;
                    Effect::Redraw
                }
                _ if search.edit(key, SEARCH_LIMIT) => {
                    self.search_changed(report);
                    Effect::Redraw
                }
                _ => Effect::Idle,
            });
        }
        if self.filter.is_some() {
            return Some(match key.code {
                KeyCode::Esc => {
                    self.filter = None;
                    Effect::Redraw
                }
                KeyCode::Enter => {
                    self.apply_filter(report);
                    Effect::Redraw
                }
                _ if delta.is_some_and(|lines| self.wheel(report, lines)) => Effect::Redraw,
                _ => Effect::Idle,
            });
        }
        if control(key, 'f') || key.code == KeyCode::Char('/') {
            self.search = Some(Input::at_end(self.query.clone()));
            self.focus = Focus::List;
            return Some(Effect::Redraw);
        }
        if control(key, 'k') || control(key, 'r') || key.code == KeyCode::Char(':') {
            return None;
        }
        if key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
        {
            return Some(Effect::Idle);
        }
        Some(match key.code {
            KeyCode::Esc if self.focus == Focus::Detail => {
                self.focus = Focus::List;
                Effect::Redraw
            }
            KeyCode::Esc => return None,
            KeyCode::Enter => {
                self.open_selected(report);
                Effect::Redraw
            }
            KeyCode::Tab | KeyCode::BackTab if self.split && self.finding(report).is_some() => {
                self.focus = if self.focus == Focus::List {
                    Focus::Detail
                } else {
                    Focus::List
                };
                Effect::Redraw
            }
            KeyCode::Char('j') => {
                let raw = serde_json::to_string_pretty(report).unwrap_or_default();
                self.hits.clear();
                self.json = Some(JsonView {
                    text: Buffer::from_streams(runner::sanitize(raw.as_bytes()), "", None),
                    scroll: 0,
                    limit: 0,
                });
                Effect::Redraw
            }
            KeyCode::Char('f') => {
                let selected = self
                    .status
                    .and_then(|status| STATUS_ORDER.iter().position(|choice| *choice == status))
                    .map_or(0, |index| index + 1);
                self.hits.clear();
                self.filter = Some(FilterMenu {
                    kind: FilterKind::Status,
                    selected,
                });
                Effect::Redraw
            }
            KeyCode::Char('r') => {
                let selected = self
                    .repo
                    .as_ref()
                    .and_then(|repo| self.repos.iter().position(|choice| choice == repo))
                    .map_or(0, |index| index + 1);
                self.hits.clear();
                self.filter = Some(FilterMenu {
                    kind: FilterKind::Repository,
                    selected,
                });
                Effect::Redraw
            }
            KeyCode::Char('a') => {
                self.groups_open = [true; 3];
                self.status = None;
                self.repo = None;
                self.query.clear();
                self.focus = Focus::List;
                self.refresh(report);
                Effect::Redraw
            }
            KeyCode::Char('b') => {
                self.status = (self.status != Some(Status::Blocked)).then_some(Status::Blocked);
                self.focus = Focus::List;
                self.refresh(report);
                Effect::Redraw
            }
            KeyCode::F(3) if self.focus == Focus::List => {
                let direction = if key.modifiers.contains(KeyModifiers::SHIFT) {
                    -1
                } else {
                    1
                };
                if self.wheel(report, direction) {
                    Effect::Redraw
                } else {
                    Effect::Idle
                }
            }
            _ if delta.is_some_and(|lines| self.wheel(report, lines)) => Effect::Redraw,
            _ => Effect::Idle,
        })
    }

    pub(super) fn frame(&mut self, report: &CheckReport, size: (u16, u16), vi: bool) -> Frame {
        self.hits.clear();
        let mut frame = Frame::new(size);
        let width = usize::from(size.0.saturating_sub(1));
        let height = usize::from(size.1);
        if width == 0 || height == 0 {
            return frame;
        }
        self.split = width >= 100 && height >= 24;
        let title = text(
            [
                "CIRNET RULES / System checks",
                "CIRNET RULES / Kiểm tra hệ thống",
            ],
            vi,
        );
        frame.rows[0].push(Tone::Heading, presentation::clip(title, width));
        let mut top = 1;
        let summary = [
            (
                status_tone(report.overall_status),
                presentation::overview(report, vi),
            ),
            (Tone::Plain, presentation::counts(report)),
            (Tone::Muted, presentation::scope(report, vi)),
        ];
        for (tone, content) in summary {
            for line in presentation::wrap(&content, width) {
                if top >= height.saturating_sub(4) {
                    break;
                }
                frame.rows[top].push(tone, line);
                top += 1;
            }
        }
        let footer = if self.json.is_some() {
            text(["Raw JSON · Esc Back", "JSON gốc · Esc Quay lại"], vi)
        } else if self.filter.is_some() {
            text(
                [
                    "Up/Down Select · Enter Apply · Esc Back",
                    "Lên/Xuống Chọn · Enter Áp dụng · Esc Quay lại",
                ],
                vi,
            )
        } else if self.search.is_some() {
            text(["Enter Search · Esc Close", "Enter Tìm · Esc Đóng"], vi)
        } else if width < 60 {
            text(
                [
                    "Up/Down · Enter Detail · / Find · f/r\na All · j JSON · Esc · : Command",
                    "Lên/Xuống · Enter · / Tìm · f/r\na Tất cả · j JSON · Esc · : Lệnh",
                ],
                vi,
            )
        } else if self.split {
            text(
                [
                    "Up/Down · Enter Detail · Tab Panel · / Search · f Status · r Repo · a All · j JSON · Shift+F10 Menu · Esc Back",
                    "Lên/Xuống · Enter Chi tiết · Tab Chuyển ô · / Tìm · f Trạng thái · r Repo · a Tất cả · j JSON · Shift+F10 Menu · Esc",
                ],
                vi,
            )
        } else {
            text(
                [
                    "Up/Down Select · Enter Detail · / Search · f Status · r Repo · a All · j JSON · Shift+F10 Menu · Esc Back",
                    "Lên/Xuống Chọn · Enter Chi tiết · / Tìm · f Trạng thái · r Repo · a Tất cả · j JSON · Shift+F10 Menu · Esc Quay lại",
                ],
                vi,
            )
        };
        let footer = presentation::wrap(footer, width);
        let bottom = height.saturating_sub(footer.len().min(height.saturating_sub(top)));
        for (row, line) in (bottom..height).zip(footer) {
            frame.rows[row].push(Tone::Muted, line);
        }
        let body_end = bottom.saturating_sub(usize::from(self.search.is_some()));
        if top >= body_end {
            return frame;
        }
        if let Some(search) = &self.search {
            view::input_line(
                &mut frame,
                body_end as u16,
                0,
                width,
                text(["Search > ", "Tìm > "], vi),
                search,
                "",
            );
        }
        let body = top..body_end;
        self.page = if self.focus == Focus::Detail || self.json.is_some() || self.filter.is_some() {
            body.len()
        } else {
            (body.len() / 3).max(1)
        };
        if let Some(json) = &mut self.json {
            json.limit = json.text.layout(width).saturating_sub(body.len());
            json.scroll = json.scroll.min(json.limit);
            for (offset, row) in body.enumerate() {
                if let Some(line) = json.text.line(json.scroll + offset, "", None) {
                    frame.rows[row] = line;
                }
            }
            return frame;
        }
        if self.filter.is_some() {
            self.draw_filter(&mut frame, body, width, vi);
            return frame;
        }
        frame.readable = Some(top as u16..body_end as u16);
        self.draw_results(&mut frame, report, body, width, vi);
        frame
    }

    fn draw_filter(&mut self, frame: &mut Frame, body: Range<usize>, width: usize, vi: bool) {
        let filter = self.filter.as_ref().expect("filter is active");
        let heading = match filter.kind {
            FilterKind::Status => text(["Filter by status", "Lọc trạng thái"], vi),
            FilterKind::Repository => text(["Filter by repository", "Lọc repository"], vi),
        };
        frame.rows[body.start].push(Tone::Heading, presentation::clip(heading, width));
        let capacity = body.len().saturating_sub(1);
        let first = filter.selected.saturating_sub(capacity.saturating_sub(1));
        let limit = filter_limit(filter.kind, &self.repos);
        for (row, index) in (body.start + 1..body.end).zip(first..=limit) {
            let choice = if index == 0 {
                text(["All", "Tất cả"], vi)
            } else {
                match filter.kind {
                    FilterKind::Status => STATUS_ORDER[index - 1].label(),
                    FilterKind::Repository => &self.repos[index - 1],
                }
            };
            let selected = filter.selected == index;
            let value = format!(
                "{} {}",
                if selected { ">" } else { " " },
                presentation::text(choice)
            );
            frame.rows[row].push(
                if selected { Tone::Accent } else { Tone::Plain },
                presentation::clip(&value, width),
            );
            self.hits.push(Hit {
                rows: row as u16..row as u16 + 1,
                columns: 0..width as u16,
                kind: HitKind::Filter(index),
            });
        }
    }

    fn draw_results(
        &mut self,
        frame: &mut Frame,
        report: &CheckReport,
        body: Range<usize>,
        width: usize,
        vi: bool,
    ) {
        if self.split {
            let left = width * 45 / 100;
            let right = width - left - 3;
            let list = self.list(report, left, body.clone(), vi);
            let details = self.details(report, right, body.len(), vi);
            self.hits.push(Hit {
                rows: body.start as u16..body.end as u16,
                columns: (left + 3) as u16..width as u16,
                kind: HitKind::Detail,
            });
            for (offset, row) in body.enumerate() {
                draw_panel(&mut frame.rows[row], list.get(offset), left, true);
                frame.rows[row].push(Tone::Muted, " │ ");
                draw_panel(&mut frame.rows[row], details.get(offset), right, false);
            }
        } else if self.focus == Focus::Detail {
            let details = self.details(report, width, body.len(), vi);
            for (row, line) in body.zip(&details) {
                draw_panel(&mut frame.rows[row], Some(line), width, false);
            }
        } else {
            let list = self.list(report, width, body.clone(), vi);
            for (row, line) in body.zip(&list) {
                draw_panel(&mut frame.rows[row], Some(line), width, false);
            }
        }
    }

    fn list(
        &mut self,
        report: &CheckReport,
        width: usize,
        body: Range<usize>,
        vi: bool,
    ) -> Vec<PanelLine> {
        if self.entries.is_empty() {
            return vec![(
                Tone::Muted,
                text(["No matching findings.", "Không có mục phù hợp."], vi).into(),
            )];
        }
        if self.selected < self.first {
            self.first = self.selected;
        }
        let capacity = body.len();
        let filtering =
            self.status.is_some() || self.repo.is_some() || !self.query.trim().is_empty();
        let filter_line = filtering && capacity > 1;
        let items_capacity = capacity - usize::from(filter_line);
        let mut used = 0;
        // Only count entries within the current viewport of the selection.
        for index in (self.first..=self.selected).rev() {
            used += self.entry_lines(report, index, width, vi).len();
            if used > items_capacity {
                self.first = (index + 1).min(self.selected);
                break;
            }
        }
        let mut lines = Vec::new();
        if filter_line {
            let status = self
                .status
                .map_or(text(["All", "Tất cả"], vi), Status::label);
            let repo = self.repo.as_deref().unwrap_or(text(["All", "Tất cả"], vi));
            let summary = format!(
                "{} {status} · Repo {repo} · {}",
                text(["Filter", "Lọc"], vi),
                presentation::text(&self.query)
            );
            lines.push((Tone::Muted, presentation::clip(&summary, width).into()));
        }
        for index in self.first..self.entries.len() {
            if lines.len() >= capacity {
                break;
            }
            let first = body.start + lines.len();
            let item = self.entry_lines(report, index, width, vi);
            lines.extend(item.into_iter().take(capacity - lines.len()));
            let end = body.start + lines.len();
            self.hits.push(Hit {
                rows: first as u16..end as u16,
                columns: 0..width as u16,
                kind: HitKind::Entry(index),
            });
        }
        lines
    }

    fn entry_lines(
        &self,
        report: &CheckReport,
        index: usize,
        width: usize,
        vi: bool,
    ) -> Vec<PanelLine> {
        let cursor = if self.selected == index { ">" } else { " " };
        match self.entries[index] {
            Entry::Group(group, count) => {
                let open = self.groups_open[group.index()]
                    || self.status.is_some()
                    || self.repo.is_some()
                    || !self.query.trim().is_empty();
                let group = format!(
                    "{cursor} {} {} ({count})",
                    if open { "[-]" } else { "[+]" },
                    group.label(vi)
                );
                vec![(
                    if self.selected == index {
                        Tone::Accent
                    } else {
                        Tone::Heading
                    },
                    presentation::clip(&group, width).into(),
                )]
            }
            Entry::Finding(index) => {
                let row = &report.findings[index];
                let title = format!(
                    "{cursor}{} {} {}",
                    presentation::marker(row.status),
                    row.status.label(),
                    presentation::title(row)
                );
                let mut lines = presentation::wrap(&title, width)
                    .into_iter()
                    .take(2)
                    .map(|line| (status_tone(row.status), line))
                    .collect::<Vec<_>>();
                let inner = width.saturating_sub(4);
                for reason in presentation::wrap(&presentation::text(&row.message), inner)
                    .into_iter()
                    .take(2)
                {
                    lines.push((Tone::Plain, format!("    {reason}")));
                }
                if let Some(action) = row.actions.first() {
                    let next = format!(
                        "{} {}",
                        text(["Next:", "Tiếp theo:"], vi),
                        presentation::text(&action.label)
                    );
                    for line in presentation::wrap(&next, inner).into_iter().take(2) {
                        lines.push((Tone::Muted, format!("    {line}")));
                    }
                }
                lines
            }
        }
    }

    fn details(
        &mut self,
        report: &CheckReport,
        width: usize,
        height: usize,
        vi: bool,
    ) -> Vec<PanelLine> {
        let lines = self
            .finding(report)
            .map(|row| presentation::detail_lines(row, width, vi));
        let Some(lines) = lines else {
            self.detail_limit = 0;
            return vec![(
                Tone::Muted,
                text(
                    [
                        "Select a finding to see details.",
                        "Chọn một mục để xem chi tiết.",
                    ],
                    vi,
                )
                .into(),
            )];
        };
        let caption = if self.focus == Focus::Detail {
            text(["> Details", "> Chi tiết"], vi)
        } else {
            text(["Details", "Chi tiết"], vi)
        };
        let window = height.saturating_sub(1);
        self.detail_limit = lines.len().saturating_sub(window);
        self.detail_scroll = self.detail_scroll.min(self.detail_limit);
        let mut panel = vec![(Tone::Heading, caption.into())];
        panel.extend(
            lines
                .into_iter()
                .enumerate()
                .skip(self.detail_scroll)
                .take(window)
                .map(|(index, line)| {
                    (
                        if index == 0 {
                            Tone::Accent
                        } else {
                            Tone::Plain
                        },
                        line,
                    )
                }),
        );
        panel
    }
}

fn same_entry(left: Entry, right: Entry) -> bool {
    match (left, right) {
        (Entry::Group(a, _), Entry::Group(b, _)) => a == b,
        (Entry::Finding(a), Entry::Finding(b)) => a == b,
        _ => false,
    }
}

fn matches_query(row: &Finding, query: &str) -> bool {
    let matches = |value: &str| presentation::text(value).to_lowercase().contains(query);
    [&row.id, &row.component, &row.code, &row.message]
        .into_iter()
        .any(|value| matches(value))
        || matches(row.status.label())
        || [
            &row.repository,
            &row.task_id,
            &row.binding_id,
            &row.agent_id,
            &row.evidence_level,
        ]
        .into_iter()
        .flatten()
        .any(|value| matches(value))
        || row.actions.iter().any(|action| matches(&action.label))
        || row.sources.iter().any(|source| matches(source))
}

fn filter_limit(kind: FilterKind, repos: &[String]) -> usize {
    match kind {
        FilterKind::Status => STATUS_ORDER.len(),
        FilterKind::Repository => repos.len(),
    }
}

fn scroll(value: &mut usize, limit: usize, lines: isize) -> bool {
    let previous = *value;
    *value = value.saturating_add_signed(lines).min(limit);
    previous != *value
}

fn status_tone(status: Status) -> Tone {
    match status {
        Status::Error => Tone::Error,
        Status::Warning | Status::Blocked | Status::Cancelled => Tone::Warning,
        Status::Pass => Tone::Success,
        Status::NotRun | Status::Unsupported | Status::Skipped => Tone::Muted,
    }
}

fn draw_panel(line: &mut Line, row: Option<&PanelLine>, width: usize, padded: bool) {
    let (tone, value) = row.map_or((Tone::Plain, ""), |(tone, value)| {
        (*tone, presentation::clip(value, width))
    });
    line.push(tone, value);
    if padded {
        line.push(Tone::Plain, " ".repeat(width.saturating_sub(value.width())));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn report(count: usize) -> CheckReport {
        let mut rows = Vec::new();
        let mut counts = std::collections::BTreeMap::new();
        for index in 0..count {
            let status = [
                Status::Pass,
                Status::NotRun,
                Status::Warning,
                Status::Blocked,
                Status::Error,
            ][index % 5];
            *counts.entry(status).or_insert(0) += 1;
            rows.push(json!({
                "id": format!("finding-{index}"),
                "component": "bindings",
                "repository": if index % 2 == 0 { "cirnet-fpga" } else { "cirnet-hardware" },
                "task_id": format!("TASK-{index}"),
                "status": status,
                "required": status == Status::Error,
                "code": format!("CODE_{index}"),
                "message": format!("Việt 🇻🇳 漢字 e\u{301} lý do {index} https://example.test/{}", "x".repeat(150)),
                "expected": "A confirmed checkout",
                "observed": { "found": false, "session_token": "fixture-confidential" },
                "evidence_level": "STATIC_VALIDATED",
                "sources": ["en/AGENTS.md"],
                "blocked_by": [],
                "duration_ms": index,
                "actions": [{"id":"select","label":"Select a confirmed worktree","operation":"select",
                    "typed_arguments":{},"effect":"read_only","requires_confirmation":false,"stale_when":[]}]
            }));
        }
        serde_json::from_value(json!({
            "schema_version": 1,
            "run_id": "fixture",
            "requested_scope": {"target":"all"},
            "resolved_context": {},
            "started_at": 0,
            "duration_ms": 4451,
            "completion": "COMPLETE",
            "overall_status": "ERROR",
            "findings": rows,
            "counts_by_status": counts,
            "coverage": {"total":count},
            "exit_code": 1
        }))
        .unwrap()
    }

    fn press(view: &mut CheckView, report: &CheckReport, code: KeyCode) {
        assert!(
            view.key(report, KeyEvent::new(code, KeyModifiers::NONE))
                .is_some()
        );
    }

    fn frame_text(frame: &Frame) -> String {
        frame
            .rows
            .iter()
            .map(Line::text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn attention_is_first_and_passed_and_pending_expand_without_altering_the_report() {
        let report = report(41);
        let original = serde_json::to_value(&report).unwrap();
        let mut view = CheckView::new(&report);
        assert_eq!(view.finding(&report).unwrap().status, Status::Error);
        let text = frame_text(&view.frame(&report, (120, 30), false));
        assert!(text.contains("Overall ERROR"));
        assert!(text.contains("41 findings"));
        assert!(text.contains("BLOCKED 8"));
        assert!(text.contains("Next: Select"));
        assert!(!text.contains("Component | Status"));
        assert!(
            view.entries
                .iter()
                .any(|entry| matches!(entry, Entry::Group(Group::Pending, 8)))
        );
        assert!(
            view.entries
                .iter()
                .any(|entry| matches!(entry, Entry::Group(Group::Passed, 9)))
        );
        assert_eq!(
            view.entries
                .iter()
                .filter(|entry| matches!(entry, Entry::Finding(_)))
                .count(),
            24
        );
        press(&mut view, &report, KeyCode::Char('a'));
        assert_eq!(
            view.entries
                .iter()
                .filter(|entry| matches!(entry, Entry::Finding(_)))
                .count(),
            41
        );
        assert_eq!(serde_json::to_value(report).unwrap(), original);
    }

    #[test]
    fn filters_search_and_raw_json_keep_finding_selection() {
        let report = report(13);
        let mut view = CheckView::new(&report);
        press(&mut view, &report, KeyCode::Char('f'));
        press(&mut view, &report, KeyCode::Down);
        press(&mut view, &report, KeyCode::Down);
        press(&mut view, &report, KeyCode::Enter);
        assert_eq!(view.status, Some(Status::Blocked));
        press(&mut view, &report, KeyCode::Char('r'));
        press(&mut view, &report, KeyCode::Down);
        press(&mut view, &report, KeyCode::Enter);
        assert_eq!(view.copy_value(&report).as_deref(), Some("TASK-8"));
        press(&mut view, &report, KeyCode::Char('/'));
        for ch in "Việt".chars() {
            press(&mut view, &report, KeyCode::Char(ch));
        }
        press(&mut view, &report, KeyCode::Enter);
        press(&mut view, &report, KeyCode::Enter);
        assert!(view.focus == Focus::Detail);
        let details = view.finding_text(&report, false).unwrap();
        assert!(details.contains("CODE_8"));
        assert!(details.contains("Evidence: STATIC_VALIDATED"));
        assert!(details.contains("Source: en/AGENTS.md"));
        assert!(details.contains("Observed:"));
        assert!(details.contains("[redacted]"));
        assert!(!details.contains("fixture-confidential"));
        press(&mut view, &report, KeyCode::Char('j'));
        assert!(view.is_json());
        assert!(view.frame(&report, (40, 12), false).readable.is_none());
        press(&mut view, &report, KeyCode::Esc);
        assert!(view.focus == Focus::Detail);
        assert_eq!(view.query, "Việt");
        assert_eq!(view.copy_value(&report).as_deref(), Some("TASK-8"));
        press(&mut view, &report, KeyCode::Esc);
        assert!(view.focus == Focus::List);
    }

    #[test]
    fn responsive_frames_bound_width_and_preserve_unicode_focus_across_sizes() {
        let report = report(135);
        let mut view = CheckView::new(&report);
        press(&mut view, &report, KeyCode::Char('a'));
        press(&mut view, &report, KeyCode::End);
        let selected = view.copy_value(&report);
        assert!(selected.is_some());
        for size in [(40, 12), (60, 20), (80, 24), (120, 30), (160, 40), (40, 12)] {
            let frame = view.frame(&report, size, true);
            assert_eq!(frame.rows.len(), usize::from(size.1));
            assert!(
                frame
                    .rows
                    .iter()
                    .all(|row| row.text().width() < usize::from(size.0))
            );
            let rendered = frame_text(&frame);
            assert!(rendered.contains("Tổng thể ERROR"));
            assert!(rendered.contains("PASS 27"));
            assert_eq!(view.copy_value(&report), selected);
        }
        press(&mut view, &report, KeyCode::Enter);
        let frame = view.frame(&report, (40, 12), true);
        assert!(frame_text(&frame).contains("Chi tiết"));
        assert!(view.wheel(&report, isize::MAX));
        let detail = view.finding_text(&report, true).unwrap();
        assert!(detail.contains("Việt 🇻🇳 漢字 e\u{301}"));
        assert!(detail.contains("Tiếp theo: Select a confirmed worktree"));
        assert!(detail.contains(&"x".repeat(60)));
    }

    #[test]
    fn mouse_hits_use_the_wrapped_visible_item_and_detail_panel() {
        let report = report(12);
        let mut view = CheckView::new(&report);
        let frame = view.frame(&report, (120, 30), false);
        let first_finding = view
            .hits
            .iter()
            .find(|hit| matches!(hit.kind, HitKind::Entry(1)))
            .unwrap();
        assert!(first_finding.rows.len() > 1);
        let wrapped_reason = first_finding.rows.start + 1;
        assert!(view.click(&report, 2, wrapped_reason));
        assert_eq!(view.finding(&report).unwrap().status, Status::Error);
        let readable = frame.readable.unwrap();
        assert!(view.click(&report, 90, readable.start));
        assert!(view.focus == Focus::Detail);
        assert!(!view.click(&report, 0, 0));
    }
}
