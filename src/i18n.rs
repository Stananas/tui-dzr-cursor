use crate::config::Language;

/// Lightweight, zero-allocation localisation. All strings are `&'static str`
/// embedded in the binary, so the i18n layer costs a few KB of static data and
/// no heap memory at all.
#[derive(Debug, Clone, Copy)]
pub struct L10n {
    pub is_fr: bool,
    // Sidebar / navigation
    pub menu: &'static str,
    pub playlists: &'static str,
    pub queue: &'static str,
    pub nav_home: &'static str,
    pub nav_explore: &'static str,
    pub nav_favorites: &'static str,
    pub nav_settings: &'static str,
    // Headers
    pub settings: &'static str,
    pub settings_subtitle: &'static str,
    pub search_results: &'static str,
    pub search: &'static str,
    pub tracks_tab: &'static str,
    pub playlists_tab: &'static str,
    pub artists_tab: &'static str,
    pub top_playlists: &'static str,
    pub top_artists: &'static str,
    pub home_title: &'static str,
    pub home_subtitle: &'static str,
    pub explore_title: &'static str,
    pub explore_subtitle: &'static str,
    pub playlist_label: &'static str,
    pub title_home: &'static str,
    pub title_explore: &'static str,
    pub title_tracks: &'static str,
    pub title_controls: &'static str,
    pub welcome: &'static str,
    // Settings items
    pub set_crossfade: &'static str,
    pub set_crossfade_duration: &'static str,
    pub set_quality: &'static str,
    pub set_discord_rpc: &'static str,
    pub set_arl: &'static str,
    pub set_language: &'static str,
    pub on: &'static str,
    pub off: &'static str,
    // Lists
    pub play_playlist: &'static str,
    pub no_results: &'static str,
    pub how_to_use: &'static str,
    pub key_navigate: &'static str,
    pub key_focus: &'static str,
    pub key_select: &'static str,
    pub key_play: &'static str,
    pub key_search: &'static str,
    pub key_quit: &'static str,
    // Player
    pub shuffle: &'static str,
    pub prev: &'static str,
    pub play: &'static str,
    pub pause: &'static str,
    pub next: &'static str,
    pub repeat: &'static str,
    pub repeat_off: &'static str,
    pub repeat_all: &'static str,
    pub repeat_one: &'static str,
    pub no_track: &'static str,
    pub seek_hint: &'static str,
    // Status messages
    pub status_waiting: &'static str,
    pub loading_home: &'static str,
    pub loading_explore: &'static str,
    pub loading_favorites: &'static str,
    pub loading_tracks: &'static str,
    pub playlists_loaded: &'static str,
    pub no_tracks_found: &'static str,
    pub searching: &'static str,
    pub queue_shuffled: &'static str,
    pub arl_updated: &'static str,
    pub type_arl_first: &'static str,
    pub flac_seek_disabled: &'static str,
    pub queue_finished: &'static str,
    pub theme_label: &'static str,
    pub playing_queue_item: &'static str,
    pub crossfading_item: &'static str,
    pub error_track_unavailable: &'static str,
    // First-run setup
    pub setup_language_title: &'static str,
    pub setup_language_hint: &'static str,
    pub setup_language_choice: &'static str,
    pub setup_language_invalid: &'static str,
    pub setup_language_done: &'static str,
    pub setup_no_arl: &'static str,
    pub setup_invalid_arl: &'static str,
    pub setup_enter_arl: &'static str,
    pub setup_arl_help: &'static str,
    pub setup_arl_failed: &'static str,
    pub setup_arl_saved: &'static str,
}

impl L10n {
    pub fn for_language(lang: Language) -> &'static L10n {
        match lang {
            Language::Fr => &FR,
            Language::En => &EN,
        }
    }

    fn pick(&self, fr: &'static str, en: &'static str) -> &'static str {
        if self.is_fr {
            fr
        } else {
            en
        }
    }

    pub fn queue_title(&self, count: usize) -> String {
        format!("{} ({})", self.queue, count)
    }

    pub fn tracks_count(&self, n: usize) -> String {
        format!("{} {}", n, self.pick("pistes recommandées", "recommended tracks"))
    }

    pub fn explore_count(&self, n: usize) -> String {
        format!("{} {}", n, self.pick("pistes à explorer", "explore tracks"))
    }

    pub fn playlist_header(&self, name: &str, id: &str) -> String {
        format!("{}: {}({})", self.playlist_label, name, id)
    }

    pub fn track_count(&self, n: usize) -> String {
        format!("{} {}", n, self.pick("pistes", "tracks"))
    }

    pub fn search_stats(&self, tracks: usize, playlists: usize, artists: usize) -> String {
        if self.is_fr {
            format!(
                "Pistes: {}  Playlists: {}  Artistes: {}",
                tracks, playlists, artists
            )
        } else {
            format!(
                "Tracks: {}  Playlists: {}  Artists: {}",
                tracks, playlists, artists
            )
        }
    }

    pub fn top_playlists_value(&self, names: &str) -> String {
        format!("{}: {}", self.top_playlists, names)
    }

    pub fn top_artists_value(&self, names: &str) -> String {
        format!("{}: {}", self.top_artists, names)
    }

    pub fn search_title(&self, tracks: usize, playlists: usize, artists: usize) -> String {
        if self.is_fr {
            format!(
                "Recherche: {} pistes, {} playlists, {} artistes",
                tracks, playlists, artists
            )
        } else {
            format!(
                "Search: {} tracks, {} playlists, {} artists",
                tracks, playlists, artists
            )
        }
    }

    pub fn crossfade_on(&self, state: &str) -> String {
        format!("{}: [{}]", self.set_crossfade, state)
    }

    pub fn crossfade_duration_value(&self, ms: u64) -> String {
        format!("{}: {}ms", self.set_crossfade_duration, ms)
    }

    pub fn quality_value(&self, label: &str) -> String {
        format!("{}: [{}]", self.set_quality, label)
    }

    pub fn discord_rpc_value(&self, state: &str) -> String {
        format!("{}: [{}]", self.set_discord_rpc, state)
    }

    pub fn language_value(&self, label: &str) -> String {
        format!("{}: [{}]", self.set_language, label)
    }

    pub fn volume(&self, percent: u16) -> String {
        format!("Vol: {}%", percent)
    }

    pub fn loaded_tracks(&self, n: usize) -> String {
        if self.is_fr {
            format!("{} pistes chargées", n)
        } else {
            format!("Loaded {} tracks", n)
        }
    }

    pub fn seek(&self, seconds: u64) -> String {
        if self.is_fr {
            format!("Rechercher: {}s", seconds)
        } else {
            format!("Seek: {}s", seconds)
        }
    }

    pub fn repeat_mode(&self, mode: &str) -> String {
        format!("{}: {}", self.repeat, mode)
    }

    pub fn theme_switched(&self, name: &str) -> String {
        format!("{}: {}", self.theme_label, name)
    }

    pub fn loading_playlist(&self, title: &str) -> String {
        if self.is_fr {
            format!("Chargement de la playlist: {}", title)
        } else {
            format!("Loading playlist: {}", title)
        }
    }

    pub fn searching_artist(&self, name: &str) -> String {
        if self.is_fr {
            format!("Recherche de l'artiste: {}", name)
        } else {
            format!("Searching artist: {}", name)
        }
    }

    pub fn playing_item(&self, index: usize, total: usize) -> String {
        format!("{} {}/{}", self.playing_queue_item, index, total)
    }

    pub fn crossfading_item(&self, index: usize, total: usize) -> String {
        format!("{} {}/{}", self.crossfading_item, index, total)
    }
}

pub static FR: L10n = L10n {
    is_fr: true,
    menu: "Menu",
    playlists: "Playlists",
    queue: "File",
    nav_home: "Accueil",
    nav_explore: "Explorer",
    nav_favorites: "Favoris",
    nav_settings: "Réglages",
    settings: "Réglages",
    settings_subtitle: "Personnalisez votre expérience d'écoute",
    search_results: "Résultats de recherche",
    search: "Recherche",
    tracks_tab: "Pistes",
    playlists_tab: "Playlists",
    artists_tab: "Artistes",
    top_playlists: "Meilleures playlists",
    top_artists: "Meilleurs artistes",
    home_title: "Accueil - Recommandé pour vous",
    home_subtitle: "Pistes personnalisées depuis votre compte Deezer",
    explore_title: "Explorer - Tendances et découvertes",
    explore_subtitle: "Recommandations en direct depuis votre session Deezer",
    playlist_label: "Playlist",
    title_home: "Accueil",
    title_explore: "Explorer",
    title_tracks: "Pistes",
    title_controls: "Contrôles",
    welcome: "Bienvenue - utilisez les raccourcis ci-dessous pour commencer",
    set_crossfade: "Fondu enchaîné",
    set_crossfade_duration: "Durée du fondu",
    set_quality: "Qualité",
    set_discord_rpc: "Discord RPC",
    set_arl: "Définir l'ARL depuis le champ de recherche",
    set_language: "Langue",
    on: "Activé",
    off: "Désactivé",
    play_playlist: "[ Lire la playlist ]",
    no_results: "Aucun résultat dans cette catégorie",
    how_to_use: "Comment utiliser",
    key_navigate: "Flèches - Naviguer",
    key_focus: "TAB - Changer de focus",
    key_select: "Entrée - Sélectionner",
    key_play: "P - Lecture/Pause",
    key_search: "/ - Rechercher",
    key_quit: "Q - Quitter",
    shuffle: "Aléatoire",
    prev: "Préc",
    play: "Lecture",
    pause: "Pause",
    next: "Suiv",
    repeat: "Répétition",
    repeat_off: "Désactivée",
    repeat_all: "Toutes",
    repeat_one: "Une",
    no_track: "Aucune piste",
    seek_hint: "< rechercher >",
    status_waiting: "Statut: En attente...",
    loading_home: "Chargement des recommandations Accueil...",
    loading_explore: "Chargement des recommandations Explorer...",
    loading_favorites: "Chargement des favoris...",
    loading_tracks: "Chargement des pistes...",
    playlists_loaded: "Playlists chargées !",
    no_tracks_found: "Aucune piste trouvée pour cette playlist/recherche",
    searching: "Recherche en cours...",
    queue_shuffled: "File mélangée",
    arl_updated: "ARL mis à jour",
    type_arl_first: "Saisissez d'abord le nouvel ARL dans le champ de recherche",
    flac_seek_disabled: "La recherche dans le FLAC est désactivée",
    queue_finished: "File terminée",
    theme_label: "Thème",
    playing_queue_item: "Lecture de l'élément",
    crossfading_item: "Fondu vers l'élément",
    error_track_unavailable: "Piste indisponible (lecture bloquée dans votre région)",
    setup_language_title: "Choisissez votre langue / Choose your language:",
    setup_language_hint: "Entrez 1 pour Français, 2 pour English, puis Entrée.",
    setup_language_choice: "Votre choix [1-2]",
    setup_language_invalid: "Choix invalide, veuillez entrer 1 ou 2.",
    setup_language_done: "Langue définie sur Français.",
    setup_no_arl: "Aucun ARL trouvé dans la configuration. Veuillez saisir votre ARL Deezer.",
    setup_invalid_arl: "L'ARL enregistré est invalide. Veuillez saisir un nouvel ARL.",
    setup_enter_arl: "Saisissez votre ARL Deezer",
    setup_arl_help: "Où trouver l'ARL : connectez-vous sur deezer.com, appuyez sur F12, puis Application → Cookies → www.deezer.com → copiez la valeur du cookie 'arl'.",
    setup_arl_failed: "Échec de la vérification de l'ARL",
    setup_arl_saved: "ARL enregistré et vérifié.",
};

pub static EN: L10n = L10n {
    is_fr: false,
    menu: "Menu",
    playlists: "Playlists",
    queue: "Queue",
    nav_home: "Home",
    nav_explore: "Explore",
    nav_favorites: "Favorites",
    nav_settings: "Settings",
    settings: "Settings",
    settings_subtitle: "Customize your playback experience",
    search_results: "Search Results",
    search: "Search",
    tracks_tab: "Tracks",
    playlists_tab: "Playlists",
    artists_tab: "Artists",
    top_playlists: "Top playlists",
    top_artists: "Top artists",
    home_title: "Home - Recommended For You",
    home_subtitle: "Tracks personalized from your Deezer account",
    explore_title: "Explore - Trending And Discovery",
    explore_subtitle: "Live recommendations from your Deezer account session",
    playlist_label: "Playlist",
    title_home: "Home",
    title_explore: "Explore",
    title_tracks: "Tracks",
    title_controls: "Controls",
    welcome: "Welcome - use the shortcuts below to get started",
    set_crossfade: "Crossfade",
    set_crossfade_duration: "Crossfade Duration",
    set_quality: "Quality",
    set_discord_rpc: "Discord RPC",
    set_arl: "Set ARL from search input",
    set_language: "Language",
    on: "On",
    off: "Off",
    play_playlist: "[ Play Playlist ]",
    no_results: "No results in this category",
    how_to_use: "How To Use",
    key_navigate: "Arrow Keys - Navigate",
    key_focus: "TAB - Switch Focus",
    key_select: "Enter - Select",
    key_play: "P - Play/Pause",
    key_search: "/ - Search",
    key_quit: "Q - Quit",
    shuffle: "Shuffle",
    prev: "Prev",
    play: "Play",
    pause: "Pause",
    next: "Next",
    repeat: "Repeat",
    repeat_off: "Off",
    repeat_all: "All",
    repeat_one: "One",
    no_track: "No track",
    seek_hint: "< seek >",
    status_waiting: "Status: Waiting...",
    loading_home: "Loading Home recommendations...",
    loading_explore: "Loading Explore recommendations...",
    loading_favorites: "Loading favorites...",
    loading_tracks: "Loading tracks...",
    playlists_loaded: "Playlists loaded!",
    no_tracks_found: "No tracks found for this playlist/search",
    searching: "Searching...",
    queue_shuffled: "Queue shuffled",
    arl_updated: "ARL updated",
    type_arl_first: "Type new ARL in search box first",
    flac_seek_disabled: "FLAC seek is disabled",
    queue_finished: "Queue finished",
    theme_label: "Theme",
    playing_queue_item: "Playing queue item",
    crossfading_item: "Crossfading to queue item",
    error_track_unavailable: "Track unavailable (playback blocked in your region)",
    setup_language_title: "Choose your language:",
    setup_language_hint: "Enter 1 for Français, 2 for English, then press Enter.",
    setup_language_choice: "Your choice [1-2]",
    setup_language_invalid: "Invalid choice, please enter 1 or 2.",
    setup_language_done: "Language set to English.",
    setup_no_arl: "No ARL found in config. Please enter your Deezer ARL.",
    setup_invalid_arl: "Saved ARL is invalid. Please enter a new ARL.",
    setup_enter_arl: "Enter Deezer ARL",
    setup_arl_help: "Where to find your ARL: log in at deezer.com, press F12, then Application → Cookies → www.deezer.com → copy the value of the 'arl' cookie.",
    setup_arl_failed: "ARL verification failed",
    setup_arl_saved: "ARL saved and verified.",
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Language;

    #[test]
    fn both_languages_resolve() {
        for lang in [Language::Fr, Language::En] {
            let i18n = L10n::for_language(lang);
            assert!(!i18n.menu.is_empty());
            assert!(!i18n.playlists.is_empty());
            assert!(!i18n.key_quit.is_empty());
        }
    }

    #[test]
    fn format_helpers_produce_translated_output() {
        let fr = L10n::for_language(Language::Fr);
        assert_eq!(fr.loaded_tracks(3), "3 pistes chargées");
        assert_eq!(fr.seek(12), "Rechercher: 12s");

        let en = L10n::for_language(Language::En);
        assert_eq!(en.loaded_tracks(3), "Loaded 3 tracks");
        assert_eq!(en.seek(12), "Seek: 12s");
    }
}