use bevy::prelude::*;
use bevy::audio::{PlaybackMode, PlaybackSettings};

#[derive(Resource, Default)]
pub struct MusicState {
    pub playing: bool,
    pub entity: Option<Entity>,
}

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicState>()
            .add_systems(Startup, start_lobby_music)
            .add_systems(Update, toggle_music_with_m);
    }
}

fn start_lobby_music(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut state: ResMut<MusicState>,
) {
    // Use OGG tracks provided in assets/music
    let tracks = vec!["music/lobbymusic1.ogg", "music/lobbymusic2.ogg"];

    // Try loading the first available track; loop it
    for path in tracks {
        let handle: Handle<AudioSource> = asset_server.load(path);
        let entity = commands
            .spawn((
                AudioPlayer(handle),
                PlaybackSettings {
                    mode: PlaybackMode::Loop,
                    ..default()
                },
            ))
            .id();
        state.playing = true;
        state.entity = Some(entity);
        return;
    }
}

fn toggle_music_with_m(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sinks: Query<&mut bevy::audio::AudioSink>,
    mut state: ResMut<MusicState>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        if let Some(entity) = state.entity {
            if let Ok(mut sink) = sinks.get_mut(entity) {
                if state.playing {
                    sink.pause();
                } else {
                    sink.play();
                }
                state.playing = !state.playing;
            }
        }
    }
}
