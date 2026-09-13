use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Padding, Paragraph},
};

use crate::{
    app::{ActivePanel, App, ClickAction, ClickRegion, RepeatMode, SearchCategory},
    config::{AudioQuality, ThemeColors},
};

use image::DynamicImage;

/// Render a DynamicImage into `area` using Unicode half-block characters (▄).
/// Works in every terminal. Each terminal row holds two image pixel-rows via
/// foreground / background colour, giving a 12×6 cell area a visually square
/// appearance with standard 8×16 px terminal fonts.
fn render_cover_art(f: &mut ratatui::Frame<'_>, img: &DynamicImage, area: ratatui::layout::Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let px_w = area.width as u32;
    let px_h = area.height as u32 * 2; // 2 pixel-rows per character row
    let resized = img.resize_exact(px_w, px_h, image::imageops::FilterType::Lanczos3);
    let rgb = resized.to_rgb8();
    let mut lines: Vec<Line<'static>> = Vec::with_capacity(area.height as usize);
    for row in 0..area.height {
        let mut spans: Vec<Span<'static>> = Vec::with_capacity(area.width as usize);
        for col in 0..area.width {
            let top = rgb.get_pixel(col as u32, (row * 2) as u32);
            let bot = rgb.get_pixel(col as u32, (row * 2 + 1) as u32);
            spans.push(Span::styled(
                "▄",
                Style::default()
                    .fg(Color::Rgb(bot[0], bot[1], bot[2]))
                    .bg(Color::Rgb(top[0], top[1], top[2])),
            ));
        }
        lines.push(Line::from(spans));
    }
    f.render_widget(Paragraph::new(lines), area);
}

fn quality_label(quality: AudioQuality) -> &'static str {
    match quality {
        AudioQuality::Kbps128 => "128kbps",
        AudioQuality::Kbps320 => "320kbps",
        AudioQuality::Flac => "FLAC",
    }
}

fn get_border_style(app: &App, panel: ActivePanel, colors: &ThemeColors) -> Style {
    if app.active_panel == panel {
        Style::default().fg(colors.accent)
    } else {
        Style::default().fg(colors.dim)
    }
}

fn panel_is_active(app: &App, panel: ActivePanel) -> bool {
    app.active_panel == panel
}

pub fn render(
    f: &mut ratatui::Frame<'_>,
    app: &mut App,
    use_true_image_protocol: bool,
) -> Option<ratatui::layout::Rect> {
    let colors = app.config.theme.colors();
    let accent = colors.accent;
    app.click_regions.clear();

    // Root layout: main content + player bar
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(6)])
        .split(f.size());

    // Workspace: sidebar + main content
    let workspace = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(32), Constraint::Min(0)])
        .split(root[0]);

    // Sidebar layout
    let sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
            Constraint::Length(1),
        ])
        .split(workspace[0]);

    let highlight_style = Style::default()
        .fg(colors.highlight_fg)
        .bg(colors.highlight_bg)
        .add_modifier(Modifier::BOLD);

    // Navigation menu
    let nav_items = vec![
        ListItem::new(app.i18n.nav_home),
        ListItem::new(app.i18n.nav_explore),
        ListItem::new(app.i18n.nav_favorites),
        ListItem::new(app.i18n.nav_settings),
    ];

    let nav_list = List::new(nav_items)
        .style(Style::default().fg(colors.fg))
        .highlight_style(highlight_style)
        .highlight_symbol(" > ")
        .block(
            Block::default()
                .title(app.i18n.menu)
                .borders(Borders::ALL)
                .border_style(get_border_style(app, ActivePanel::Navigation, &colors))
                .padding(Padding::new(1, 0, 0, 0)),
        );
    f.render_stateful_widget(nav_list, sidebar[0], &mut app.nav_state);
    app.click_regions.push(ClickRegion {
        rect: sidebar[0],
        action: ClickAction::NavList,
    });

    // Playlists list
    let playlist_items: Vec<ListItem<'_>> = app
        .playlists
        .iter()
        .map(|(_, title)| ListItem::new(title.as_str()))
        .collect();

    let playlist_list = List::new(playlist_items)
        .style(Style::default().fg(colors.fg))
        .highlight_style(highlight_style)
        .highlight_symbol(" > ")
        .block(
            Block::default()
                .title(app.i18n.playlists)
                .borders(Borders::ALL)
                .border_style(get_border_style(app, ActivePanel::Playlists, &colors))
                .padding(Padding::new(1, 0, 0, 0)),
        );
    f.render_stateful_widget(playlist_list, sidebar[1], &mut app.playlist_state);
    app.click_regions.push(ClickRegion {
        rect: sidebar[1],
        action: ClickAction::PlaylistList,
    });

    // Queue list
    let queue_items: Vec<ListItem<'_>> = app
        .queue
        .iter()
        .map(|name| ListItem::new(name.as_str()))
        .collect();

    let queue_list = List::new(queue_items)
        .style(Style::default().fg(colors.fg))
        .highlight_style(highlight_style)
        .highlight_symbol(" > ")
        .block(
            Block::default()
                .title(app.i18n.queue_title(app.queue.len()))
                .borders(Borders::ALL)
                .border_style(get_border_style(app, ActivePanel::Queue, &colors))
                .padding(Padding::new(1, 0, 0, 0)),
        );
    f.render_stateful_widget(queue_list, sidebar[2], &mut app.queue_state);
    app.click_regions.push(ClickRegion {
        rect: sidebar[2],
        action: ClickAction::QueueList,
    });

    // Status bar
    let status_bar = Paragraph::new(app.status_message.as_str())
        .style(Style::default().fg(colors.status))
        .alignment(Alignment::Left);
    f.render_widget(status_bar, sidebar[3]);

    // Main content area layout: search bar + main content
    let main_sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(workspace[1]);

    // Search bar
    let search_bar_text = if app.is_searching {
        format!("🔍 {}_", app.search_query)
    } else {
        format!("🔍 {}", app.search_query)
    };

    let search_border_style = if app.is_searching || panel_is_active(app, ActivePanel::Search) {
        Style::default().fg(colors.accent)
    } else {
        Style::default().fg(colors.dim)
    };

    let search_bar = Paragraph::new(search_bar_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(search_border_style)
                .padding(Padding::new(1, 1, 0, 0)),
        )
        .style(Style::default().fg(colors.fg));
    f.render_widget(search_bar, main_sections[0]);
    app.click_regions.push(ClickRegion {
        rect: main_sections[0],
        action: ClickAction::SearchBox,
    });

    // Main content sections: header + list
    let main_content_sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(main_sections[1]);

    // Main view header
    let is_home_page = app.current_playlist_id.as_deref() == Some("__home__");
    let is_explore_page = app.current_playlist_id.as_deref() == Some("__explore__");

    let header = if app.viewing_settings {
        Paragraph::new(vec![
            Line::from(Span::styled(
                app.i18n.settings,
                Style::default().fg(colors.fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                app.i18n.settings_subtitle,
                Style::default().fg(colors.dim),
            )),
        ])
    } else if app.showing_search_results {
        let tracks_tab = if app.search_category == SearchCategory::Tracks {
            format!("[{}]", app.i18n.tracks_tab)
        } else {
            format!(" {} ", app.i18n.tracks_tab)
        };
        let playlists_tab = if app.search_category == SearchCategory::Playlists {
            format!("[{}]", app.i18n.playlists_tab)
        } else {
            format!(" {} ", app.i18n.playlists_tab)
        };
        let artists_tab = if app.search_category == SearchCategory::Artists {
            format!("[{}]", app.i18n.artists_tab)
        } else {
            format!(" {} ", app.i18n.artists_tab)
        };

        Paragraph::new(vec![
            Line::from(Span::styled(
                app.i18n.search_results,
                Style::default().fg(colors.fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("{}  {}  {}", tracks_tab, playlists_tab, artists_tab),
                Style::default().fg(colors.accent).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                app.i18n.search_stats(
                    app.current_tracks.len(),
                    app.search_playlists.len(),
                    app.search_artists.len()
                ),
                Style::default().fg(colors.dim),
            )),
            Line::from(Span::styled(
                app.i18n.top_playlists_value(
                    &app.search_playlists
                        .iter()
                        .take(2)
                        .map(|(_, t)| t.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Style::default().fg(colors.dim),
            )),
            Line::from(Span::styled(
                app.i18n.top_artists_value(
                    &app.search_artists
                        .iter()
                        .take(2)
                        .map(|(_, a)| a.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Style::default().fg(colors.dim),
            )),
        ])
    } else if is_home_page {
        Paragraph::new(vec![
            Line::from(Span::styled(
                app.i18n.home_title,
                Style::default().fg(colors.fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                app.i18n.home_subtitle,
                Style::default().fg(colors.dim),
            )),
            Line::from(Span::styled(
                app.i18n.tracks_count(app.current_tracks.len()),
                Style::default().fg(colors.dim),
            )),
        ])
    } else if is_explore_page {
        Paragraph::new(vec![
            Line::from(Span::styled(
                app.i18n.explore_title,
                Style::default().fg(colors.fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                app.i18n.explore_subtitle,
                Style::default().fg(colors.dim),
            )),
            Line::from(Span::styled(
                app.i18n.explore_count(app.current_tracks.len()),
                Style::default().fg(colors.dim),
            )),
        ])
    } else if !app.current_tracks.is_empty() {
        let playlist_id = app.current_playlist_id.as_deref().unwrap_or("Unknown");
        let playlist_name = app
            .current_playlist_id
            .as_ref()
            .and_then(|id| {
                app.playlists
                    .iter()
                    .find(|(pid, _)| pid == id)
                    .map(|(_, title)| title.as_str())
            })
            .or_else(|| {
                app.search_playlists
                    .iter()
                    .find(|(pid, _)| Some(pid) == app.current_playlist_id.as_ref())
                    .map(|(_, title)| title.as_str())
            })
            .unwrap_or(app.i18n.playlist_label);

        Paragraph::new(vec![
            Line::from(Span::styled(
                app.i18n.playlist_header(playlist_name, playlist_id),
                Style::default().fg(colors.fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                app.i18n.track_count(app.current_tracks.len()),
                Style::default().fg(colors.dim),
            )),
        ])
    } else {
        Paragraph::new(vec![
            Line::from(Span::styled(
                "Deezer-TUI",
                Style::default().fg(colors.fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                app.i18n.welcome,
                Style::default().fg(colors.dim),
            )),
        ])
    };

    let main_title = if app.viewing_settings {
        app.i18n.settings.to_string()
    } else if is_home_page {
        app.i18n.title_home.to_string()
    } else if is_explore_page {
        app.i18n.title_explore.to_string()
    } else if app.showing_search_results {
        match app.search_category {
            SearchCategory::Tracks => format!("{}: {}", app.i18n.search, app.i18n.tracks_tab),
            SearchCategory::Playlists => format!("{}: {}", app.i18n.search, app.i18n.playlists_tab),
            SearchCategory::Artists => format!("{}: {}", app.i18n.search, app.i18n.artists_tab),
        }
    } else if !app.current_tracks.is_empty() {
        app.i18n.title_tracks.to_string()
    } else {
        app.i18n.title_controls.to_string()
    };

    let header_block = header.block(
        Block::default()
            .title(main_title)
            .borders(Borders::ALL)
            .border_style(get_border_style(app, ActivePanel::Main, &colors))
            .padding(Padding::new(2, 2, 1, 0)),
    );
    f.render_widget(header_block, main_content_sections[0]);

    // Main content area
    if app.viewing_settings {
        // Settings view
        let on_off = |v: bool| if v { app.i18n.on } else { app.i18n.off };
        let settings_items = vec![
            app.i18n.crossfade_on(on_off(app.config.crossfade_enabled)),
            app.i18n.crossfade_duration_value(app.config.crossfade_duration_ms),
            app.i18n.quality_value(quality_label(app.config.default_quality)),
            app.i18n.discord_rpc_value(on_off(app.discord_rpc_enabled)),
            app.i18n.set_arl.to_string(),
            app.i18n.language_value(app.config.language.label()),
        ];

        let settings_list_items: Vec<ListItem> = settings_items
            .iter()
            .map(|item| ListItem::new(item.as_str()))
            .collect();

        let settings_list = List::new(settings_list_items)
            .style(Style::default().fg(colors.fg))
            .highlight_style(highlight_style)
            .highlight_symbol(" > ")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.dim))
                    .padding(Padding::new(1, 1, 0, 0)),
            );

        f.render_stateful_widget(settings_list, main_content_sections[1], &mut app.settings_state);
        app.click_regions.push(ClickRegion {
            rect: main_content_sections[1],
            action: ClickAction::SettingsList,
        });
    } else if app.showing_search_results {
        // Tracks list view
        let track_items: Vec<ListItem> = match app.search_category {
            SearchCategory::Tracks => {
                let mut items: Vec<ListItem> = vec![ListItem::new(app.i18n.play_playlist)];
                items.extend(app.current_tracks.iter().map(|(_, title, artist)| {
                    ListItem::new(format!("{} - {}", title, artist))
                }));
                items
            }
            SearchCategory::Playlists => app
                .search_playlists
                .iter()
                .map(|(_, title)| ListItem::new(format!("{}", title)))
                .collect(),
            SearchCategory::Artists => app
                .search_artists
                .iter()
                .map(|(_, name)| ListItem::new(format!("{}", name)))
                .collect(),
        };

        let empty_hint = track_items.is_empty();

        let tracks_list = List::new(track_items)
            .style(Style::default().fg(colors.fg))
            .highlight_style(highlight_style)
            .highlight_symbol(" > ")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.dim))
                    .padding(Padding::new(1, 1, 0, 0)),
            );

        if empty_hint {
            f.render_widget(
                Paragraph::new(app.i18n.no_results)
                    .style(Style::default().fg(colors.dim))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(colors.dim))
                            .padding(Padding::new(1, 1, 0, 0)),
                    ),
                main_content_sections[1],
            );
        } else {
            f.render_stateful_widget(tracks_list, main_content_sections[1], &mut app.main_state);
        }
        app.click_regions.push(ClickRegion {
            rect: main_content_sections[1],
            action: ClickAction::MainList,
        });
    } else if !app.current_tracks.is_empty() {
        let mut track_items: Vec<ListItem> = vec![ListItem::new(app.i18n.play_playlist)];
        track_items.extend(app.current_tracks.iter().map(|(_, title, artist)| {
            ListItem::new(format!("{} - {}", title, artist))
        }));

        let tracks_list = List::new(track_items)
            .style(Style::default().fg(colors.fg))
            .highlight_style(highlight_style)
            .highlight_symbol(" > ")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.dim))
                    .padding(Padding::new(1, 1, 0, 0)),
            );

        f.render_stateful_widget(tracks_list, main_content_sections[1], &mut app.main_state);
        app.click_regions.push(ClickRegion {
            rect: main_content_sections[1],
            action: ClickAction::MainList,
        });
    } else {
        // Startup controls page
        let controls = Paragraph::new(vec![
            Line::from(Span::styled(
                app.i18n.key_navigate,
                Style::default().fg(colors.fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(app.i18n.key_focus, Style::default().fg(colors.dim))),
            Line::from(Span::styled(app.i18n.key_select, Style::default().fg(colors.dim))),
            Line::from(Span::styled(app.i18n.key_play, Style::default().fg(colors.dim))),
            Line::from(Span::styled(app.i18n.key_search, Style::default().fg(colors.dim))),
            Line::from(Span::styled(app.i18n.key_quit, Style::default().fg(colors.dim))),
        ])
        .block(
            Block::default()
                .title(app.i18n.how_to_use)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.dim))
                .padding(Padding::new(2, 2, 1, 1)),
        );
        f.render_widget(controls, main_content_sections[1]);
    }

    // Player bar layout
    let player_bar = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(root[1]);

    let (track_title, track_artist, current_ms, total_ms) = match &app.now_playing {
        Some(now) => (
            now.title.clone(),
            now.artist.clone(),
            now.current_ms,
            now.total_ms.max(1),
        ),
        None => (
            app.i18n.no_track.to_owned(),
            "-".to_owned(),
            0,
            1,
        ),
    };

    let track_info = Paragraph::new(vec![
        Line::from(Span::styled(
            track_title,
            Style::default().fg(colors.fg).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(track_artist, Style::default().fg(colors.dim))),
    ]);
    // ── bottom-left panel: cover art thumbnail + track title/artist ──────────
    // ART_COLS is chosen so that with 8×16 px terminal cells the displayed image
    // is visually square (12 cols × 8 px = 96 px wide; 6 rows × 16 px = 96 px tall).
    const ART_COLS: u16 = 12;
    let show_art = app.cover_art.is_some() && player_bar[0].width > ART_COLS + 8;
    let mut protocol_art_rect: Option<ratatui::layout::Rect> = None;
    if show_art {
        let pb0_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(ART_COLS), Constraint::Min(0)])
            .split(player_bar[0]);
        if use_true_image_protocol {
            f.render_widget(
                Block::default().borders(Borders::ALL).border_style(Style::default().fg(colors.dim)),
                pb0_split[0],
            );
            protocol_art_rect = Some(
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
                    .split(pb0_split[0])[1],
            );
        } else if let Some(img) = &app.cover_art {
            render_cover_art(f, img, pb0_split[0]);
        }
        let info = track_info.clone().block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.dim))
                .padding(Padding::new(1, 1, 0, 0)),
        );
        f.render_widget(info, pb0_split[1]);
    } else {
        let info = track_info.block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.dim))
                .padding(Padding::new(1, 1, 0, 0)),
        );
        f.render_widget(info, player_bar[0]);
    }

    let controls_and_progress = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(2)])
        .split(player_bar[1]);

    let play_symbol = if app.is_playing {
        app.i18n.pause
    } else {
        app.i18n.play
    };
    let repeat_label = match app.repeat_mode {
        RepeatMode::Off => app.i18n.repeat_off,
        RepeatMode::All => app.i18n.repeat_all,
        RepeatMode::One => app.i18n.repeat_one,
    };
    let labels = [
        app.i18n.shuffle.to_string(),
        app.i18n.prev.to_string(),
        play_symbol.to_string(),
        app.i18n.next.to_string(),
        app.i18n.repeat_mode(repeat_label),
    ];

    let mut control_spans = Vec::new();
    for (i, label) in labels.iter().enumerate() {
        let active = app.active_panel == ActivePanel::Player && app.player_button_index == i;
        let style = if active {
            Style::default().fg(colors.fg).bg(colors.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors.fg)
        };
        control_spans.push(Span::styled(format!("[{}]", label), style));
        control_spans.push(Span::raw("  "));
    }

    let controls = Paragraph::new(Line::from(control_spans))
        .alignment(Alignment::Center)
        .style(if app.active_panel == ActivePanel::Player {
            Style::default().fg(colors.fg)
        } else {
            Style::default().fg(colors.dim)
        });
    f.render_widget(controls, controls_and_progress[0]);

    // Register clickable regions for the player control buttons. Widths are
    // derived from the rendered `[label]  ` spans so clicks line up visually.
    let total_btn_width: u16 = labels
        .iter()
        .map(|l| (l.chars().count() + 2) as u16)
        .sum::<u16>()
        .saturating_add(2 * (labels.len().saturating_sub(1)) as u16);
    let mut x_cursor = controls_and_progress[0]
        .x
        .saturating_add(controls_and_progress[0].width.saturating_sub(total_btn_width) / 2);
    for (i, label) in labels.iter().enumerate() {
        let btn_width = (label.chars().count() + 2) as u16;
        if btn_width > 0 {
            app.click_regions.push(ClickRegion {
                rect: Rect::new(x_cursor, controls_and_progress[0].y, btn_width, 1),
                action: ClickAction::PlayerButton(i),
            });
        }
        x_cursor = x_cursor.saturating_add(btn_width + 2);
    }

    let ratio = (current_ms as f64 / total_ms as f64).clamp(0.0, 1.0);
    let seeking_active = app.active_panel == ActivePanel::PlayerProgress;

    // Split progress row into: current time | gauge | total time
    let progress_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(5), Constraint::Min(0), Constraint::Length(5)])
        .split(controls_and_progress[1]);

    let cur_min = current_ms / 60_000;
    let cur_sec = (current_ms / 1_000) % 60;
    let tot_min = total_ms / 60_000;
    let tot_sec = (total_ms / 1_000) % 60;

    let time_style = if seeking_active {
        Style::default().fg(colors.accent_alt).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(colors.dim)
    };

    f.render_widget(
        Paragraph::new(format!("{:02}:{:02}", cur_min, cur_sec))
            .style(time_style)
            .alignment(Alignment::Right),
        progress_row[0],
    );

    let gauge = Gauge::default()
        .style(Style::default().fg(colors.dim))
        .gauge_style(
            if seeking_active {
                Style::default().fg(colors.accent_alt).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(accent).add_modifier(Modifier::BOLD)
            }
        )
        .use_unicode(true)
        .ratio(ratio)
        .label(if seeking_active { app.i18n.seek_hint } else { "" });
    f.render_widget(gauge, progress_row[1]);
    app.click_regions.push(ClickRegion {
        rect: progress_row[1],
        action: ClickAction::SeekBar,
    });

    f.render_widget(
        Paragraph::new(format!("{:02}:{:02}", tot_min, tot_sec))
            .style(time_style)
            .alignment(Alignment::Left),
        progress_row[2],
    );

    let vol_selected = app.active_panel == ActivePanel::PlayerInfo;
    let volume_settings = Paragraph::new(Line::from(Span::styled(
        app.i18n.volume(app.volume),
        if vol_selected {
            Style::default().fg(colors.fg).bg(colors.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors.dim)
        },
    )))
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(get_border_style(app, ActivePanel::PlayerInfo, &colors))
                .padding(Padding::new(1, 1, 1, 0)),
        );
    f.render_widget(volume_settings, player_bar[2]);
    // Clickable volume row. The block has a top padding of 1, so content
    // starts two rows below the border.
    app.click_regions.push(ClickRegion {
        rect: Rect::new(player_bar[2].x, player_bar[2].y + 2, player_bar[2].width, 1),
        action: ClickAction::VolumeLine,
    });

    protocol_art_rect
}

#[cfg(test)]
mod tests {
    use super::*;
use crate::app::App;
use crate::config::{Config, Theme};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;

    fn test_app() -> App {
        let (tx, _rx) = mpsc::unbounded_channel();
        App::new(Config::default(), tx)
    }

    #[test]
    fn render_populates_click_regions() {
        let mut app = test_app();
        app.playlists
            .push(("1".to_string(), "Playlist A".to_string()));
        app.current_tracks
            .push(("t1".to_string(), "Title".to_string(), "Artist".to_string()));

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                render(f, &mut app, false);
            })
            .unwrap();

        assert!(!app.click_regions.is_empty());
        for action in [
            ClickAction::NavList,
            ClickAction::PlaylistList,
            ClickAction::QueueList,
            ClickAction::SearchBox,
            ClickAction::MainList,
            ClickAction::SeekBar,
            ClickAction::VolumeLine,
        ] {
            assert!(
                app.click_regions.iter().any(|r| r.action == action),
                "missing click region for {action:?}"
            );
        }
        let buttons: Vec<usize> = app
            .click_regions
            .iter()
            .filter_map(|r| match r.action {
                ClickAction::PlayerButton(i) => Some(i),
                _ => None,
            })
            .collect();
        assert_eq!(buttons, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn theme_cycles_through_all_variants() {
        let mut theme = Theme::Deezer;
        let variants = [Theme::Deezer, Theme::SpotifyDark, Theme::NcmpcppBlue];
        for expected in variants {
            assert_eq!(theme, expected);
            theme = theme.next();
        }
    }

    #[test]
    fn deezer_theme_accent_applied_to_active_border() {
        let mut app = test_app();
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                render(f, &mut app, false);
            })
            .unwrap();
        let buf = terminal.backend().buffer();
        // Top-left corner of the active "Menu" block (Navigation is the default
        // active panel) must use the Deezer accent purple.
        let corner = buf.get(0, 0);
        assert_eq!(
            corner.style().fg,
            Some(Color::Rgb(0xA2, 0x38, 0xFF)),
            "active border should use Deezer accent"
        );
    }

    #[test]
    fn render_dump_deezer_theme() {
        let mut app = test_app();
        app.playlists
            .push(("1".to_string(), "My Playlist".to_string()));
        app.playlists
            .push(("2".to_string(), "Discover Weekly".to_string()));
        app.current_tracks
            .push(("t1".to_string(), "Starboy".to_string(), "The Weeknd".to_string()));
        app.current_tracks
            .push(("t2".to_string(), "Blinding Lights".to_string(), "The Weeknd".to_string()));
        app.now_playing = Some(crate::app::NowPlaying {
            id: "t1".to_string(),
            title: "Starboy".to_string(),
            artist: "The Weeknd".to_string(),
            quality: AudioQuality::Kbps320,
            current_ms: 45_000,
            total_ms: 200_000,
            album_art_url: None,
        });
        app.is_playing = true;
        app.main_state.select(Some(1));

        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                render(f, &mut app, false);
            })
            .unwrap();

        let buf = terminal.backend().buffer();
        for y in 0..buf.area.height {
            let mut line = String::new();
            for x in 0..buf.area.width {
                line.push_str(buf.get(x, y).symbol());
            }
            println!("{:02}|{}|", y, line);
        }
    }
}
