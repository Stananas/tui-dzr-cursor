use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use tokio::sync::mpsc;

use crate::config::{AudioQuality, Config};
use crate::i18n::L10n;

use anyhow::{anyhow, Result as AnyResult};
use mpris_server::LoopStatus;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickAction {
    NavList,
    PlaylistList,
    QueueList,
    MainList,
    SettingsList,
    SearchBox,
    PlayerButton(usize),
    SeekBar,
    VolumeLine,
}

#[derive(Debug, Clone, Copy)]
pub struct ClickRegion {
    pub rect: Rect,
    pub action: ClickAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnterOutcome {
    None,
    SaveConfig,
    MprisLoopUpdate(LoopStatus),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    PlayTrack(String),
    AutoPlayTrack(String),
    PlayTrackAt {
        track_id: String,
        quality: AudioQuality,
        seek_ms: u64,
    },
    Pause,
    Resume,
    Next,
    Previous,
    SetVolume(u16),
    SetQuality(AudioQuality),
    SetCrossfade { enabled: bool, duration_ms: u64 },
    ToggleCrossfade,
    LoadPlaylist(String),
    LoadHome,
    LoadExplore,
    LoadFavorites,
    Search(String),
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiEvent {
    PlaybackProgress {
        current_ms: u64,
        total_ms: u64,
    },
    TrackChanged {
        id: String,
        title: String,
        artist: String,
        quality: AudioQuality,
        album_art_url: Option<String>,
        initial_ms: u64,
    },
    PlaybackPaused,
    PlaybackResumed,
    PlaybackStopped,
    Error(String),
    /// Informational status (shown in the status bar), distinct from errors.
    Status(String),
    PlaylistsLoaded(Vec<(String, String)>),
    TracksLoaded(Vec<(String, String, String)>),
    SearchResultsLoaded {
        tracks: Vec<(String, String, String)>,
        playlists: Vec<(String, String)>,
        artists: Vec<(String, String)>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchCategory {
    Tracks,
    Playlists,
    Artists,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Library,
    Search,
    Queue,
    Lyrics,
    Settings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NowPlaying {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub quality: AudioQuality,
    pub current_ms: u64,
    pub total_ms: u64,
    pub album_art_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    Navigation,
    Playlists,
    Queue,
    Search,
    Main,
    Player,
    PlayerProgress,
    PlayerInfo,
}

#[derive(Debug, Clone)]
pub struct App {
    pub config: Config,
    pub i18n: &'static L10n,
    pub command_sender: mpsc::UnboundedSender<Command>,
    pub current_route: Route,
    pub now_playing: Option<NowPlaying>,
    pub is_playing: bool,
    pub volume: u16,
    pub discord_rpc_enabled: bool,
    pub active_panel: ActivePanel,
    pub nav_state: ListState,
    pub playlist_state: ListState,
    pub queue_state: ListState,
    pub playlists: Vec<(String, String)>,
    pub queue: Vec<String>,
    pub queue_tracks: Vec<(String, String, String)>,
    pub queue_index: Option<usize>,
    pub current_tracks: Vec<(String, String, String)>,
    pub search_playlists: Vec<(String, String)>,
    pub search_artists: Vec<(String, String)>,
    pub showing_search_results: bool,
    pub search_category: SearchCategory,
    pub main_state: ListState,
    pub settings_state: ListState,
    pub player_button_index: usize,
    pub repeat_mode: RepeatMode,
    pub viewing_settings: bool,
    pub current_playlist_id: Option<String>,
    pub status_message: String,
    pub is_searching: bool,
    pub search_query: String,
    pub auto_transition_armed: bool,
    /// Decoded cover-art image for the current track (None until downloaded).
    pub cover_art: Option<image::DynamicImage>,
    pub cover_art_png: Option<Vec<u8>>,
    pub cover_art_track_id: Option<String>,
    /// Mouse-clickable regions recomputed every frame by the UI renderer.
    pub click_regions: Vec<ClickRegion>,
}

impl App {
    pub fn new(config: Config, command_sender: mpsc::UnboundedSender<Command>) -> Self {
        let mut nav_state = ListState::default();
        nav_state.select(Some(0));

        let mut playlist_state = ListState::default();
        playlist_state.select(Some(0));

        let mut queue_state = ListState::default();
        queue_state.select(Some(0));

        let mut main_state = ListState::default();
        main_state.select(Some(0));

        let mut settings_state = ListState::default();
        settings_state.select(Some(0));

        let discord_rpc_enabled = config.discord_rpc_enabled;
        let i18n = L10n::for_language(config.language);

        Self {
            volume: 100,
            config,
            i18n,
            command_sender,
            current_route: Route::Library,
            now_playing: None,
            is_playing: false,
            discord_rpc_enabled,
            active_panel: ActivePanel::Navigation,
            nav_state,
            playlist_state,
            queue_state,
            playlists: vec![],
            queue: vec![],
            queue_tracks: vec![],
            queue_index: None,
            current_tracks: vec![],
            search_playlists: vec![],
            search_artists: vec![],
            showing_search_results: false,
            search_category: SearchCategory::Tracks,
            main_state,
            settings_state,
            player_button_index: 2,
            repeat_mode: RepeatMode::Off,
            viewing_settings: false,
            current_playlist_id: None,
            status_message: i18n.status_waiting.into(),
            is_searching: false,
            search_query: String::new(),
            auto_transition_armed: false,
            cover_art: None,
            cover_art_png: None,
            cover_art_track_id: None,
            click_regions: vec![],
        }
    }

    pub fn handle_down(&mut self) {
        match self.active_panel {
            ActivePanel::Navigation => {
                let current = self.nav_state.selected().unwrap_or(0);
                if current >= 3 {
                    self.active_panel = ActivePanel::Playlists;
                    if !self.playlists.is_empty() {
                        self.playlist_state.select(Some(0));
                    }
                } else {
                    self.nav_state.select(Some(current + 1));
                }
            }
            ActivePanel::Playlists => {
                if self.playlists.is_empty() {
                    self.active_panel = ActivePanel::Queue;
                    if !self.queue.is_empty() {
                        self.queue_state.select(Some(0));
                    }
                    return;
                }

                let max = self.playlists.len() - 1;
                let current = self.playlist_state.selected().unwrap_or(0);
                if current >= max {
                    self.active_panel = ActivePanel::Queue;
                    if !self.queue.is_empty() {
                        self.queue_state.select(Some(0));
                    }
                } else {
                    self.playlist_state.select(Some(current + 1));
                }
            }
            ActivePanel::Queue => {
                if self.queue.is_empty() {
                    return;
                }

                let max = self.queue.len() - 1;
                let current = self.queue_state.selected().unwrap_or(0);
                self.queue_state.select(Some((current + 1).min(max)));
            }
            ActivePanel::Search => {
                self.active_panel = ActivePanel::Main;
            }
            ActivePanel::Main => {
                if self.viewing_settings {
                    let max = 5usize;
                    let current = self.settings_state.selected().unwrap_or(0);
                    self.settings_state.select(Some((current + 1).min(max)));
                } else if self.showing_search_results {
                    let max = match self.search_category {
                        SearchCategory::Tracks => self.current_tracks.len(),
                        SearchCategory::Playlists => self.search_playlists.len().saturating_sub(1),
                        SearchCategory::Artists => self.search_artists.len().saturating_sub(1),
                    };
                    let current = self.main_state.selected().unwrap_or(0);
                    self.main_state.select(Some((current + 1).min(max)));
                } else if !self.current_tracks.is_empty() {
                    // +1 for the top action row: "Play Playlist"
                    let max = self.current_tracks.len();
                    let current = self.main_state.selected().unwrap_or(0);
                    self.main_state.select(Some((current + 1).min(max)));
                } else {
                    self.active_panel = ActivePanel::Player;
                }
            }
            ActivePanel::Player => {
                self.active_panel = ActivePanel::PlayerProgress;
            }
            ActivePanel::PlayerProgress => {}
            ActivePanel::PlayerInfo => {}
        }
    }

    pub fn handle_up(&mut self) {
        match self.active_panel {
            ActivePanel::Queue => {
                if self.queue.is_empty() {
                    self.active_panel = ActivePanel::Playlists;
                    if !self.playlists.is_empty() {
                        self.playlist_state.select(Some(self.playlists.len() - 1));
                    }
                    return;
                }

                let current = self.queue_state.selected().unwrap_or(0);
                if current == 0 {
                    self.active_panel = ActivePanel::Playlists;
                    if !self.playlists.is_empty() {
                        self.playlist_state.select(Some(self.playlists.len() - 1));
                    }
                } else {
                    self.queue_state.select(Some(current - 1));
                }
            }
            ActivePanel::Playlists => {
                if self.playlists.is_empty() {
                    self.active_panel = ActivePanel::Navigation;
                    self.nav_state.select(Some(3));
                    return;
                }

                let current = self.playlist_state.selected().unwrap_or(0);
                if current == 0 {
                    self.active_panel = ActivePanel::Navigation;
                    self.nav_state.select(Some(3));
                } else {
                    self.playlist_state.select(Some(current - 1));
                }
            }
            ActivePanel::Navigation => {
                let current = self.nav_state.selected().unwrap_or(0);
                self.nav_state.select(Some(current.saturating_sub(1)));
            }
            ActivePanel::Search => {
                self.active_panel = ActivePanel::Navigation;
            }
            ActivePanel::Main => {
                if self.viewing_settings {
                    let current = self.settings_state.selected().unwrap_or(0);
                    self.settings_state.select(Some(current.saturating_sub(1)));
                } else if !self.current_tracks.is_empty() {
                    let current = self.main_state.selected().unwrap_or(0);
                    if current == 0 {
                        self.active_panel = ActivePanel::Search;
                    } else {
                        self.main_state.select(Some(current.saturating_sub(1)));
                    }
                }
            }
            ActivePanel::Player => {
                self.active_panel = ActivePanel::Main;
            }
            ActivePanel::PlayerProgress => {
                self.active_panel = ActivePanel::Player;
            }
            ActivePanel::PlayerInfo => {
                self.active_panel = ActivePanel::Player;
            }
        }
    }

    pub fn handle_right(&mut self) {
        match self.active_panel {
            ActivePanel::Navigation | ActivePanel::Playlists | ActivePanel::Queue => {
                self.active_panel = ActivePanel::Search;
            }
            ActivePanel::Search => {
                self.active_panel = ActivePanel::Main;
            }
            ActivePanel::Main => {
                self.active_panel = ActivePanel::Player;
            }
            ActivePanel::Player => {
                if self.player_button_index < 4 {
                    self.player_button_index += 1;
                } else {
                    self.active_panel = ActivePanel::PlayerInfo;
                }
            }
            ActivePanel::PlayerProgress => {
                self.active_panel = ActivePanel::PlayerInfo;
            }
            ActivePanel::PlayerInfo => {}
        }
    }

    pub fn handle_left(&mut self) {
        match self.active_panel {
            ActivePanel::Main => {
                if self.showing_search_results {
                    self.search_category = match self.search_category {
                        SearchCategory::Tracks => SearchCategory::Artists,
                        SearchCategory::Playlists => SearchCategory::Tracks,
                        SearchCategory::Artists => SearchCategory::Playlists,
                    };
                    self.main_state.select(Some(0));
                } else {
                    self.active_panel = ActivePanel::Playlists;
                }
            }
            ActivePanel::Player => {
                self.player_button_index = self.player_button_index.saturating_sub(1);
            }
            ActivePanel::PlayerProgress => {
                self.active_panel = ActivePanel::Player;
            }
            ActivePanel::PlayerInfo => {
                self.active_panel = ActivePanel::Player;
                self.player_button_index = 4;
            }
            _ => {}
        }
    }

    pub fn switch_search_category_right(&mut self) {
        if self.showing_search_results && self.active_panel == ActivePanel::Main {
            self.search_category = match self.search_category {
                SearchCategory::Tracks => SearchCategory::Playlists,
                SearchCategory::Playlists => SearchCategory::Artists,
                SearchCategory::Artists => SearchCategory::Tracks,
            };
            self.main_state.select(Some(0));
        }
    }

    /// Handles the Enter key (and left-click activation). Anything that touches
    /// resources outside `App` (config persistence, MPRIS) is reported back via
    /// the returned outcome so the caller can apply it.
    pub fn handle_enter(&mut self) -> AnyResult<EnterOutcome> {
        let mut outcome = EnterOutcome::None;
        match self.active_panel {
            ActivePanel::Navigation => {
                let nav_idx = self.nav_state.selected().unwrap_or(0);
                match nav_idx {
                    0 => {
                        self.command_sender
                            .send(Command::LoadHome)
                            .map_err(|_| anyhow!("failed to send load home command"))?;
                        self.current_playlist_id = Some("__home__".to_string());
                        self.active_panel = ActivePanel::Main;
                        self.status_message = self.i18n.loading_home.into();
                    }
                    1 => {
                        self.command_sender
                            .send(Command::LoadExplore)
                            .map_err(|_| anyhow!("failed to send load explore command"))?;
                        self.current_playlist_id = Some("__explore__".to_string());
                        self.active_panel = ActivePanel::Main;
                        self.status_message = self.i18n.loading_explore.into();
                    }
                    2 => {
                        self.command_sender
                            .send(Command::LoadFavorites)
                            .map_err(|_| anyhow!("failed to send load favorites command"))?;
                        self.active_panel = ActivePanel::Main;
                        self.status_message = self.i18n.loading_favorites.into();
                    }
                    3 => {
                        self.viewing_settings = true;
                        self.active_panel = ActivePanel::Main;
                    }
                    _ => {}
                }
            }
            ActivePanel::Playlists => {
                if let Some(idx) = self.playlist_state.selected() {
                    if idx < self.playlists.len() {
                        let (playlist_id, _) = &self.playlists[idx];
                        self.current_playlist_id = Some(playlist_id.clone());
                        self.command_sender
                            .send(Command::LoadPlaylist(playlist_id.clone()))
                            .map_err(|_| anyhow!("failed to send load playlist command"))?;
                        self.active_panel = ActivePanel::Main;
                    }
                }
            }
            ActivePanel::Queue => {
                if let Some(idx) = self.queue_state.selected() {
                    if idx < self.queue_tracks.len() {
                        self.queue_index = Some(idx);
                        let (track_id, _, _) = &self.queue_tracks[idx];
                        self.command_sender
                            .send(Command::PlayTrack(track_id.clone()))
                            .map_err(|_| anyhow!("failed to play queued track"))?;
                        self.is_playing = true;
                    }
                }
            }
            ActivePanel::Main => {
                if self.viewing_settings {
                    if let Some(idx) = self.settings_state.selected() {
                        match idx {
                            0 => {
                                self.config.crossfade_enabled = !self.config.crossfade_enabled;
                                self.command_sender
                                    .send(Command::SetCrossfade {
                                        enabled: self.config.crossfade_enabled,
                                        duration_ms: self.config.crossfade_duration_ms,
                                    })
                                    .map_err(|_| anyhow!("failed to set crossfade"))?;
                                outcome = EnterOutcome::SaveConfig;
                            }
                            1 => {
                                let presets = [1000u64, 3000, 5000, 8000, 10000, 13000];
                                let current = self.config.crossfade_duration_ms;
                                let next = presets
                                    .iter()
                                    .copied()
                                    .find(|value| *value > current)
                                    .unwrap_or(presets[0]);
                                self.config.crossfade_duration_ms = next;
                                self.command_sender
                                    .send(Command::SetCrossfade {
                                        enabled: self.config.crossfade_enabled,
                                        duration_ms: self.config.crossfade_duration_ms,
                                    })
                                    .map_err(|_| anyhow!("failed to set crossfade duration"))?;
                                outcome = EnterOutcome::SaveConfig;
                            }
                            2 => {
                                self.config.default_quality = match self.config.default_quality {
                                    AudioQuality::Kbps128 => AudioQuality::Kbps320,
                                    AudioQuality::Kbps320 => AudioQuality::Flac,
                                    AudioQuality::Flac => AudioQuality::Kbps128,
                                };
                                self.command_sender
                                    .send(Command::SetQuality(self.config.default_quality))
                                    .map_err(|_| anyhow!("failed to set quality"))?;
                                outcome = EnterOutcome::SaveConfig;
                            }
                            3 => {
                                self.discord_rpc_enabled = !self.discord_rpc_enabled;
                                self.config.discord_rpc_enabled = self.discord_rpc_enabled;
                                outcome = EnterOutcome::SaveConfig;
                            }
                            4 => {
                                let new_arl = self.search_query.trim();
                                if new_arl.is_empty() {
                                    self.status_message = self.i18n.type_arl_first.into();
                                } else {
                                    let trimmed = new_arl.to_owned();
                                    self.config.arl = trimmed;
                                    self.status_message = self.i18n.arl_updated.into();
                                    self.search_query.clear();
                                    outcome = EnterOutcome::SaveConfig;
                                }
                            }
                            5 => {
                                self.config.language = self.config.language.next();
                                self.config.language_set = true;
                                self.i18n = L10n::for_language(self.config.language);
                                self.status_message =
                                    format!("{}: {}", self.i18n.set_language, self.config.language.label());
                                outcome = EnterOutcome::SaveConfig;
                            }
                            _ => {}
                        }
                    }
                } else if self.showing_search_results {
                    if let Some(idx) = self.main_state.selected() {
                        match self.search_category {
                            SearchCategory::Tracks => {
                                if idx == 0 {
                                    self.queue_tracks = self.current_tracks.clone();
                                    self.queue = self
                                        .queue_tracks
                                        .iter()
                                        .map(|(_, title, artist)| format!("{} - {}", title, artist))
                                        .collect();
                                    self.queue_state.select(Some(0));
                                    self.queue_index = Some(0);

                                    if let Some((track_id, _, _)) = self.queue_tracks.first() {
                                        self.command_sender
                                            .send(Command::PlayTrack(track_id.clone()))
                                            .map_err(|_| anyhow!("failed to send play track command"))?;
                                        self.is_playing = true;
                                    }
                                } else {
                                    let track_idx = idx - 1;
                                    if track_idx < self.current_tracks.len() {
                                        let selected = self.current_tracks[track_idx].clone();
                                        self.queue_tracks = vec![selected.clone()];
                                        self.queue = vec![format!("{} - {}", selected.1, selected.2)];
                                        self.queue_state.select(Some(0));
                                        self.queue_index = Some(0);

                                        self.command_sender
                                            .send(Command::PlayTrack(selected.0))
                                            .map_err(|_| anyhow!("failed to send play track command"))?;
                                        self.is_playing = true;
                                    }
                                }
                            }
                            SearchCategory::Playlists => {
                                if idx < self.search_playlists.len() {
                                    let (playlist_id, title) = &self.search_playlists[idx];
                                    self.current_playlist_id = Some(playlist_id.clone());
                                    self.command_sender
                                        .send(Command::LoadPlaylist(playlist_id.clone()))
                                        .map_err(|_| anyhow!("failed to load playlist from search"))?;
                                    self.status_message = self.i18n.loading_playlist(title);
                                }
                            }
                            SearchCategory::Artists => {
                                if idx < self.search_artists.len() {
                                    let (_, name) = &self.search_artists[idx];
                                    self.command_sender
                                        .send(Command::Search(name.clone()))
                                        .map_err(|_| anyhow!("failed to search artist"))?;
                                    self.status_message = self.i18n.searching_artist(name);
                                }
                            }
                        }
                    }
                } else if !self.current_tracks.is_empty() {
                    if let Some(idx) = self.main_state.selected() {
                        if idx == 0 {
                            self.queue_tracks = self.current_tracks.clone();
                            self.queue = self
                                .queue_tracks
                                .iter()
                                .map(|(_, title, artist)| format!("{} - {}", title, artist))
                                .collect();
                            self.queue_state.select(Some(0));
                            self.queue_index = Some(0);

                            if let Some((track_id, _, _)) = self.queue_tracks.first() {
                                self.command_sender
                                    .send(Command::PlayTrack(track_id.clone()))
                                    .map_err(|_| anyhow!("failed to send play track command"))?;
                                self.is_playing = true;
                            }
                        } else {
                            let track_idx = idx - 1;
                            if track_idx < self.current_tracks.len() {
                                let selected = self.current_tracks[track_idx].clone();
                                self.queue_tracks = vec![selected.clone()];
                                self.queue = vec![format!("{} - {}", selected.1, selected.2)];
                                self.queue_state.select(Some(0));
                                self.queue_index = Some(0);

                                self.command_sender
                                    .send(Command::PlayTrack(selected.0))
                                    .map_err(|_| anyhow!("failed to send play track command"))?;
                                self.is_playing = true;
                            }
                        }
                    }
                }
            }
            ActivePanel::Player => {
                match self.player_button_index {
                    0 => {
                        if !self.queue_tracks.is_empty() {
                            let current_id = self
                                .queue_index
                                .and_then(|i| self.queue_tracks.get(i))
                                .map(|t| t.0.clone());
                            self.queue_tracks.shuffle(&mut thread_rng());
                            if let Some(current) = current_id {
                                self.queue_index = self
                                    .queue_tracks
                                    .iter()
                                    .position(|t| t.0 == current);
                            }
                            self.queue = self
                                .queue_tracks
                                .iter()
                                .map(|(_, title, artist)| format!("{} - {}", title, artist))
                                .collect();
                            if let Some(i) = self.queue_index {
                                self.queue_state.select(Some(i));
                            }
                            self.status_message = self.i18n.queue_shuffled.into();
                        }
                    }
                    1 => {
                        if let Some(current_idx) = self.queue_index {
                            if current_idx > 0 {
                                let prev_idx = current_idx - 1;
                                self.queue_index = Some(prev_idx);
                                self.queue_state.select(Some(prev_idx));
                                if let Some((track_id, _, _)) = self.queue_tracks.get(prev_idx) {
                                    self.command_sender
                                        .send(Command::PlayTrack(track_id.clone()))
                                        .map_err(|_| anyhow!("failed to play previous track"))?;
                                    self.is_playing = true;
                                }
                            }
                        }
                    }
                    2 => {
                        if self.is_playing {
                            self.command_sender
                                .send(Command::Pause)
                                .map_err(|_| anyhow!("failed to pause"))?;
                        } else {
                            self.command_sender
                                .send(Command::Resume)
                                .map_err(|_| anyhow!("failed to resume"))?;
                        }
                    }
                    3 => {
                        if let Some(current_idx) = self.queue_index {
                            let next_idx = current_idx + 1;
                            if next_idx < self.queue_tracks.len() {
                                self.queue_index = Some(next_idx);
                                self.queue_state.select(Some(next_idx));
                                if let Some((track_id, _, _)) = self.queue_tracks.get(next_idx) {
                                    self.command_sender
                                        .send(Command::PlayTrack(track_id.clone()))
                                        .map_err(|_| anyhow!("failed to play next track"))?;
                                    self.is_playing = true;
                                }
                            }
                        }
                    }
                    4 => {
                        self.repeat_mode = match self.repeat_mode {
                            RepeatMode::Off => RepeatMode::All,
                            RepeatMode::All => RepeatMode::One,
                            RepeatMode::One => RepeatMode::Off,
                        };
                        let loop_status = match self.repeat_mode {
                            RepeatMode::Off => LoopStatus::None,
                            RepeatMode::One => LoopStatus::Track,
                            RepeatMode::All => LoopStatus::Playlist,
                        };
                        outcome = EnterOutcome::MprisLoopUpdate(loop_status);
                        let repeat_label = match self.repeat_mode {
                            RepeatMode::Off => self.i18n.repeat_off,
                            RepeatMode::All => self.i18n.repeat_all,
                            RepeatMode::One => self.i18n.repeat_one,
                        };
                        self.status_message = self.i18n.repeat_mode(repeat_label);
                    }
                    _ => {}
                }
            }
            ActivePanel::Search => {
                self.command_sender
                    .send(Command::Search(self.search_query.clone()))
                    .map_err(|_| anyhow!("failed to send search command"))?;
                self.active_panel = ActivePanel::Main;
            }
            ActivePanel::PlayerProgress => {}
            ActivePanel::PlayerInfo => {}
        }
        Ok(outcome)
    }
}
