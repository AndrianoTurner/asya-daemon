use tracing::*;
use macros::Stringify;
use serde::{Deserialize, Serialize};

use crate::scenarios::*;

/// Usecases are the main business logic of the application.
///
/// This usecases module contains all the possible actions that the user can perform from client.
#[derive(Debug, Stringify, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum InternalUsecases {
    TurnOffMusic,
    TurnOnMusic,
    GetMusicStatus,
    PlayNextTrack,
    PlayPrevTrack,

    #[serde(rename_all = "camelCase")]
    Open {
        app_kind: AppKind,
    },

    StartBasicSystemMonitoring,
    Answer,
}

#[derive(Serialize, Stringify, Deserialize, Debug, Clone, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AppKind {
    Terminal,
    Browser,
    Steam,
    Discord,
    Telegram,
    Specific(App),
}

#[derive(Serialize, Stringify, Deserialize, Debug, Clone, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum App {
    // Tui(String),
    Gui(String),
}

impl InternalUsecases {
    pub fn stringify_all() -> String {
        let strings = [
            InternalUsecases::stringify_one(),
            AppKind::stringify_one(),
            App::stringify_one(),
        ];
        let iter = strings.iter().map(|el| el.to_string() + "\n\n");
        String::from_iter(iter)
    }
    pub async fn execute(self, userinput: String) {
        let command = self;
        debug!("Dispatching command: {:?}", command);
        match command {
            InternalUsecases::TurnOffMusic | InternalUsecases::TurnOnMusic => {
                music_control::play_or_resume_music(userinput).await;
            }
            InternalUsecases::GetMusicStatus => {
                music_control::get_music_status(userinput).await;
            }
            InternalUsecases::PlayNextTrack => music_control::play_next_track(userinput).await,
            InternalUsecases::PlayPrevTrack => music_control::play_previous_track(userinput).await,
            InternalUsecases::StartBasicSystemMonitoring => {
                system_monitoring::start_basic_monitoring(userinput).await
            }
            InternalUsecases::Open { app_kind } => open::open(app_kind).await,
            InternalUsecases::Answer => geranal_answer::answer(userinput).await,
        }
    }
}

// if new usecases with some params will be added, they should be added as example to the `Requests` enum in `requests.rs`
