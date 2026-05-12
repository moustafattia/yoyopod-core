use anyhow::{anyhow, bail, Result};

use super::listen::sync_rows;
use super::shared::{FooterBar, StatusBarWidgets};
use super::TypedScreenController;
use crate::presentation::transitions::TransitionSampler;
use crate::presentation::view_models::{ListScreenModel, ScreenModel};
use crate::render::widgets::{roles, LvglFacade, WidgetId};
use yoyopod_protocol::ui::UiScreen;

#[derive(Default)]
pub struct PlaylistController {
    root: Option<WidgetId>,
    status: StatusBarWidgets,
    title: Option<WidgetId>,
    underline: Option<WidgetId>,
    panel: Option<WidgetId>,
    footer: FooterBar,
    row_containers: Vec<WidgetId>,
    row_icons: Vec<WidgetId>,
    row_titles: Vec<WidgetId>,
    row_subtitles: Vec<WidgetId>,
    empty_panel: Option<WidgetId>,
    empty_icon: Option<WidgetId>,
    empty_title: Option<WidgetId>,
    empty_subtitle: Option<WidgetId>,
}

#[derive(Clone, Copy)]
pub struct PlaylistControllerModel<'a> {
    pub(crate) screen: UiScreen,
    pub(crate) list: &'a ListScreenModel,
}

impl PlaylistController {
    fn ensure_widgets(&mut self, facade: &mut dyn LvglFacade, row_count: usize) -> Result<()> {
        if self.root.is_none() {
            self.root = Some(facade.create_root()?);
        }
        let root = self
            .root
            .ok_or_else(|| anyhow!("playlist controller missing root widget"))?;

        if self.title.is_none() {
            self.title = Some(facade.create_label(root, roles::PLAYLIST_TITLE)?);
        }
        if self.underline.is_none() {
            self.underline = Some(facade.create_container(root, roles::PLAYLIST_UNDERLINE)?);
        }
        if self.panel.is_none() {
            self.panel = Some(facade.create_container(root, roles::PLAYLIST_PANEL)?);
        }
        let panel = self
            .panel
            .ok_or_else(|| anyhow!("playlist controller missing panel"))?;

        while self.row_titles.len() < row_count {
            let row = facade.create_container(panel, roles::PLAYLIST_ROW)?;
            self.row_containers.push(row);
            self.row_icons
                .push(facade.create_label(row, roles::PLAYLIST_ROW_ICON)?);
            self.row_titles
                .push(facade.create_label(row, roles::PLAYLIST_ROW_TITLE)?);
            self.row_subtitles
                .push(facade.create_label(row, roles::PLAYLIST_ROW_SUBTITLE)?);
        }
        if self.empty_panel.is_none() {
            self.empty_panel = Some(facade.create_container(root, roles::PLAYLIST_EMPTY_PANEL)?);
        }
        let empty_panel = self
            .empty_panel
            .ok_or_else(|| anyhow!("playlist controller missing empty panel"))?;
        if self.empty_icon.is_none() {
            self.empty_icon = Some(facade.create_label(empty_panel, roles::PLAYLIST_EMPTY_ICON)?);
        }
        if self.empty_title.is_none() {
            self.empty_title = Some(facade.create_label(empty_panel, roles::PLAYLIST_EMPTY_TITLE)?);
        }
        if self.empty_subtitle.is_none() {
            self.empty_subtitle =
                Some(facade.create_label(empty_panel, roles::PLAYLIST_EMPTY_SUBTITLE)?);
        }

        Ok(())
    }
}

impl TypedScreenController for PlaylistController {
    const SUPPORTS_TRANSITIONS: bool = true;

    type Model<'a> = PlaylistControllerModel<'a>;

    fn model<'a>(model: &'a ScreenModel) -> Result<Self::Model<'a>> {
        playlist_model(model)
    }

    fn sync_model(
        &mut self,
        facade: &mut dyn LvglFacade,
        model: Self::Model<'_>,
        transitions: &TransitionSampler<'_>,
    ) -> Result<()> {
        let list = model.list;
        let accent = accent_for_playlist(list);

        self.ensure_widgets(facade, 4)?;
        if let Some(root) = self.root {
            self.status.sync(facade, root, &list.chrome.status, false)?;
            self.footer
                .sync(facade, root, roles::PLAYLIST_FOOTER, &list.chrome.footer)?;
        }
        if let Some(title) = self.title {
            facade.set_text(title, &list.title)?;
        }
        if let Some(underline) = self.underline {
            facade.set_accent(underline, accent)?;
        }
        let empty = list.rows.is_empty();
        if let Some(panel) = self.panel {
            facade.set_visible(panel, !empty)?;
        }
        if let Some(empty_panel) = self.empty_panel {
            facade.set_visible(empty_panel, empty)?;
        }
        if let Some(empty_icon) = self.empty_icon {
            facade.set_icon(empty_icon, playlist_empty_icon(model.screen))?;
            facade.set_accent(empty_icon, accent)?;
        }
        if let Some(empty_title) = self.empty_title {
            facade.set_text(empty_title, playlist_empty_title(model.screen))?;
        }
        if let Some(empty_subtitle) = self.empty_subtitle {
            facade.set_text(empty_subtitle, playlist_empty_subtitle(model.screen, list))?;
        }
        sync_rows(
            facade,
            list,
            accent,
            &self.row_containers,
            &self.row_icons,
            &self.row_titles,
            &self.row_subtitles,
            transitions,
        )
    }

    fn teardown(&mut self, facade: &mut dyn LvglFacade) -> Result<()> {
        let root = self.root.take();
        self.status.clear();
        self.title = None;
        self.underline = None;
        self.panel = None;
        self.footer.clear();
        self.row_containers.clear();
        self.row_icons.clear();
        self.row_titles.clear();
        self.row_subtitles.clear();
        self.empty_panel = None;
        self.empty_icon = None;
        self.empty_title = None;
        self.empty_subtitle = None;
        if let Some(root) = root {
            facade.destroy(root)?;
        }
        Ok(())
    }
}

fn playlist_model(model: &ScreenModel) -> Result<PlaylistControllerModel<'_>> {
    match model {
        ScreenModel::Playlists(list)
        | ScreenModel::RecentTracks(list)
        | ScreenModel::Contacts(list)
        | ScreenModel::CallHistory(list) => Ok(PlaylistControllerModel {
            screen: model.screen(),
            list,
        }),
        _ => bail!(
            "playlist controller received non-playlist screen model: {}",
            model.screen().as_str()
        ),
    }
}

fn accent_for_playlist(model: &ListScreenModel) -> u32 {
    match model.title.to_ascii_lowercase().as_str() {
        "contacts" | "history" | "recents" => 0x00D4FF,
        _ => 0x00FF88,
    }
}

fn playlist_empty_title(screen: UiScreen) -> &'static str {
    match screen {
        UiScreen::Contacts => "No contacts",
        UiScreen::CallHistory => "No recent calls",
        UiScreen::Playlists => "No playlists",
        UiScreen::RecentTracks => "No recent tracks",
        _ => "No items",
    }
}

fn playlist_empty_subtitle(screen: UiScreen, list: &ListScreenModel) -> &str {
    match screen {
        UiScreen::Contacts | UiScreen::CallHistory => &list.subtitle,
        UiScreen::Playlists => "Saved mixes will appear here.",
        UiScreen::RecentTracks => "Recent songs will appear here.",
        _ => &list.subtitle,
    }
}

fn playlist_empty_icon(screen: UiScreen) -> &'static str {
    match screen {
        UiScreen::Contacts | UiScreen::CallHistory => "talk",
        UiScreen::RecentTracks => "recent",
        _ => "playlist",
    }
}
