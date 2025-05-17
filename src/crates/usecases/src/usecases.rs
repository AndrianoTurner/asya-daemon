use macros::Stringify;
use serde::{Deserialize, Serialize};

use crate::scenarios::*;

/// Usecases are the main business logic of the application.
///
/// This usecases module contains all the possible actions that the user can perform from client.
#[derive(Debug, Stringify, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum InternalUsecases {
    #[schemars(description = "Turns off the music playback. Pause music.")]
    TurnOffMusic,

    #[schemars(description = "Turns on the music playback. Plays music. Returns Music.")]
    TurnOnMusic,

    #[schemars(description = "Retrieves the current music playback status")]
    GetMusicStatus,

    #[schemars(description = "Plays the next track in the music playlist")]
    PlayNextTrack,

    #[schemars(description = "Plays the previous track in the music playlist")]
    PlayPrevTrack,

    #[schemars(description = "Opens a specified application")]
    #[serde(rename_all = "camelCase")]
    Open { app_kind: AppKind },

    #[schemars(description = "Starts basic system monitoring functionality")]
    StartBasicSystemMonitoring,

    #[schemars(description = "Provides an answer to a query")]
    Answer,
}

#[derive(Serialize, Stringify, Deserialize, Debug, Clone, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AppKind {
    #[schemars(description = "Opens the terminal application")]
    Terminal,

    #[schemars(description = "Opens the web browser")]
    Browser,

    #[schemars(description = "Opens the Steam gaming platform")]
    Steam,

    #[schemars(description = "Opens the Discord application")]
    Discord,

    #[schemars(description = "Opens the Telegram messaging application")]
    Telegram,

    #[schemars(description = "Opens a specific application")]
    Specific(App),
}

#[derive(Serialize, Stringify, Deserialize, Debug, Clone, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum App {
    // #[schemars(description = "Represents a text-based user interface application with a given name")]
    // Tui(String),
    #[schemars(
        description = "Represents a graphical user interface application with a given name"
    )]
    Gui(String),
}

impl InternalUsecases {
    pub async fn dispatch(self, userinput: String) {
        let command = self;
        match command {
            
            InternalUsecases::TurnOffMusic | InternalUsecases::TurnOnMusic => {
                #[cfg(target_family = "unix")]
                music_control::play_or_resume_music(userinput).await;
            }
            InternalUsecases::GetMusicStatus => {
                #[cfg(target_family = "unix")]
                music_control::get_music_status(userinput).await;
            }
            InternalUsecases::PlayNextTrack => {
                #[cfg(target_family = "unix")]
                music_control::play_next_track(userinput).await
            },
            InternalUsecases::PlayPrevTrack => {
                #[cfg(target_family = "unix")]
                music_control::play_previous_track(userinput).await
            },
            InternalUsecases::StartBasicSystemMonitoring => {
                system_monitoring::start_basic_monitoring(userinput).await
            }
            InternalUsecases::Open { app_kind } => open::open(app_kind).await,
            InternalUsecases::Answer => geranal_answer::answer(userinput).await,
        }
    }
}

// if new usecases with some params will be added, they should be added as example to the `Requests` enum in `requests.rs`
