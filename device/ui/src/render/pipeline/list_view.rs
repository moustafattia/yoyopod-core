use crate::presentation::registry::NativeRenderScene;
use crate::presentation::view_models::{ListScreenModel, ScreenModel};

const NATIVE_LIST_VISIBLE_ROWS: usize = 4;

#[derive(Debug, Clone, Copy)]
enum NativeListSelection {
    Wrap,
    Clamp,
}

pub(super) fn rust_owned_scene_model(model: &ScreenModel, scene: NativeRenderScene) -> ScreenModel {
    match (scene, model) {
        (NativeRenderScene::Listen, ScreenModel::Listen(list)) => {
            ScreenModel::Listen(capped_list_model(list, NativeListSelection::Wrap))
        }
        (NativeRenderScene::Playlist, ScreenModel::Playlists(list)) => {
            ScreenModel::Playlists(capped_list_model(list, NativeListSelection::Clamp))
        }
        (NativeRenderScene::Playlist, ScreenModel::RecentTracks(list)) => {
            ScreenModel::RecentTracks(capped_list_model(list, NativeListSelection::Clamp))
        }
        (NativeRenderScene::Playlist, ScreenModel::Contacts(list)) => {
            ScreenModel::Contacts(capped_list_model(list, NativeListSelection::Clamp))
        }
        (NativeRenderScene::Playlist, ScreenModel::CallHistory(list)) => {
            ScreenModel::CallHistory(capped_list_model(list, NativeListSelection::Clamp))
        }
        _ => model.clone(),
    }
}

fn capped_list_model(model: &ListScreenModel, selection: NativeListSelection) -> ListScreenModel {
    let mut rows = model
        .rows
        .iter()
        .take(NATIVE_LIST_VISIBLE_ROWS)
        .cloned()
        .collect::<Vec<_>>();

    if !rows.is_empty() {
        let selected_index = model.rows.iter().position(|row| row.selected).unwrap_or(0);
        let visible_index = match selection {
            NativeListSelection::Wrap => selected_index % rows.len(),
            NativeListSelection::Clamp => selected_index.min(rows.len() - 1),
        };
        for row in &mut rows {
            row.selected = false;
        }
        rows[visible_index].selected = true;
    }

    ListScreenModel {
        chrome: model.chrome.clone(),
        title: model.title.clone(),
        subtitle: model.subtitle.clone(),
        rows,
    }
}
