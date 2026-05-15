pub mod connections;
pub mod dashboard;
pub mod logs;
pub mod proxies;
pub mod rules;
pub mod settings;
pub mod subscriptions;

use std::collections::HashMap;

use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

use super::theme::Theme;
use crate::event::action::Action;
use crate::state::store::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageId {
    Dashboard = 0,
    Proxies = 1,
    Connections = 2,
    Logs = 3,
    Rules = 4,
    Settings = 5,
    Subscriptions = 6,
}

impl PageId {
    pub fn all() -> &'static [PageId] {
        &[
            PageId::Dashboard,
            PageId::Proxies,
            PageId::Connections,
            PageId::Logs,
            PageId::Rules,
            PageId::Settings,
            PageId::Subscriptions,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            PageId::Dashboard => "Dashboard",
            PageId::Proxies => "Proxies",
            PageId::Connections => "Conns",
            PageId::Logs => "Mihomo Logs",
            PageId::Rules => "Rules",
            PageId::Settings => "Settings",
            PageId::Subscriptions => "Subs",
        }
    }

    pub fn next(self) -> Self {
        let all = Self::all();
        let pos = all.iter().position(|p| *p == self).unwrap_or(0);
        all[(pos + 1) % all.len()]
    }

    pub fn prev(self) -> Self {
        let all = Self::all();
        let pos = all.iter().position(|p| *p == self).unwrap_or(0);
        all[(pos + all.len() - 1) % all.len()]
    }
}

/// Every application page implements this trait
pub trait Page {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, state: &AppState);
    fn handle_key(&mut self, key: KeyEvent, state: &AppState) -> Vec<Action>;
    fn tick(&mut self) -> Vec<Action> {
        vec![]
    }
    fn page_id(&self) -> PageId;
    fn title(&self) -> &'static str;
    fn on_enter(&mut self) {}
    fn on_leave(&mut self) {}
    /// If true, global keybindings are suppressed (page captures all input).
    fn is_modal(&self) -> bool {
        false
    }
}

pub struct PageRouter {
    pages: HashMap<PageId, Box<dyn Page>>,
    active: PageId,
}

impl PageRouter {
    pub fn new() -> Self {
        let mut pages: HashMap<PageId, Box<dyn Page>> = HashMap::new();
        pages.insert(PageId::Dashboard, Box::new(dashboard::DashboardPage::new()));
        pages.insert(PageId::Proxies, Box::new(proxies::ProxiesPage::new()));
        pages.insert(
            PageId::Connections,
            Box::new(connections::ConnectionsPage::new()),
        );
        pages.insert(PageId::Logs, Box::new(logs::LogsPage::new()));
        pages.insert(PageId::Rules, Box::new(rules::RulesPage::new()));
        pages.insert(PageId::Settings, Box::new(settings::SettingsPage::new()));
        pages.insert(
            PageId::Subscriptions,
            Box::new(subscriptions::SubscriptionsPage::new()),
        );
        Self {
            pages,
            active: PageId::Dashboard,
        }
    }

    pub fn active_page(&self) -> &dyn Page {
        self.pages[&self.active].as_ref()
    }

    pub fn active_page_mut(&mut self) -> &mut dyn Page {
        self.pages.get_mut(&self.active).unwrap().as_mut()
    }

    pub fn active_id(&self) -> PageId {
        self.active
    }

    pub fn switch_to(&mut self, id: PageId) {
        if id != self.active {
            self.pages.get_mut(&self.active).unwrap().on_leave();
            self.active = id;
            self.pages.get_mut(&self.active).unwrap().on_enter();
        }
    }

    pub fn next_page(&mut self) {
        let next = self.active.next();
        self.switch_to(next);
    }

    pub fn prev_page(&mut self) {
        let prev = self.active.prev();
        self.switch_to(prev);
    }
}
