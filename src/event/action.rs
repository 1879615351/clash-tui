use crate::clash::RefreshData;
use crate::ui::pages::PageId;

/// All possible actions in the app
#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    NextPage,
    PrevPage,
    SwitchPage(PageId),
    ToggleHelp,
    SelectNext,
    SelectPrev,
    SelectProxy { group: String, proxy: String },
    TestLatency { group: String, proxy: String },
    TestAllLatency { group: String, proxies: Vec<String> },
    CloseConnection(String),
    CloseAllConnections,
    ToggleSystemProxy,
    EnableSystemProxy,
    DisableSystemProxy,
    DownloadSubscription(String),
    ToggleSubscription(String),
    AddSubscription { name: String, url: String },
    RemoveSubscription(String),
    SetClashMode(String),
    RestartMihomo,
    CycleTheme,
    RefreshData,
    UpdateData(RefreshData),
    Error(String),
    Noop,
}
