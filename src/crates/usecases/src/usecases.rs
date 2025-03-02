use std::{collections::HashMap, future::{Future, IntoFuture}, marker::PhantomData, pin::Pin, process::Output};

use async_trait::async_trait;
use macros::Stringify;

use shared::configuration::CONFIG;
use tokio::pin;
use tracing::*;

use crate::scenarios::*;

#[async_trait]
pub trait Usecase {
   async fn execute(&self, input: &str);
   fn enabled(&self) -> bool{
        true
   }
   fn description(&self) -> String{
        "No description_provided".into()
   }
   fn set_enabled(&mut self,enabled: bool);
   fn set_description(&mut self, description: String);
}
struct GenericUsecase<F,Fut>
where
    F: Fn(&str) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
{
    closure: F,
    enabled: bool,
    description: String,
    _phantom_data: PhantomData<Fut>,
}

impl<F,Fut> GenericUsecase<F,Fut>
where
F: Fn(&str) -> Fut + Send + Sync,
Fut: Future<Output = ()> + Send
{
    fn new(closure: F) -> Self
    {
        Self {
            closure,
            enabled: true,
            description: "".into(),
            _phantom_data: PhantomData
        }
    }
}


#[async_trait]
impl<F,Fut> Usecase for GenericUsecase<F,Fut>
where
    F: Fn(&str) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send + Sync,
    {
    async fn execute(&self, input: &str){
        (self.closure)(input).await;
    }
    
    fn set_enabled(&mut self,enabled: bool){
        self.enabled = enabled;
    }
    fn set_description(&mut self, description: String){
        self.description = description
    }
}
struct OpenApp{
    enabled: bool,
    description: String
}

impl OpenApp{
    pub fn new(enabled: bool, description: String) -> Self{
        Self { enabled, description }
    }
}

impl Default for OpenApp{
    fn default() -> Self {
        Self { enabled: true, description: "".into() }
    }
}

#[async_trait]
impl Usecase for OpenApp {
    async fn execute(&self, input: &str) { 
        open_app::open(input.into()).await;
    }
    fn set_enabled(&mut self,enabled: bool){
        self.enabled = enabled;
    }
    fn set_description(&mut self, description: String){
        self.description = description
    }
}

/// Usecases are the main business logic of the application.
///
/// This usecases module contains all the possible actions that the user can perform from client.
#[derive(Debug, Stringify, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Usecases {
    /// Turns off current track.
    /// # Examples
    ///  - Asya, turns off the music, please.
    ///  - Shut up music
    TurnOffMusic,

    /// Turns on current track.
    /// # Examples
    ///  - Asya, turn the music back on.
    ///  - Resume the song
    TurnOnMusic,

    /// Returns currently playing track.
    ///
    /// # Examples
    ///  - What song is playing right now?
    ///  - What's the name of the current track?
    GetMusicStatus,

    /// Play next track.
    ///
    /// # Examples
    ///  - Next song, please.
    ///  - Skip to the next track.
    PlayNextTrack,

    /// Play previous track.
    ///
    /// # Examples
    ///  - Play the previous song.
    ///  - Go back to the last track.
    PlayPrevTrack,

    /// Turns off the computer.
    Shutdown,

    /// Reboot computer.
    Reboot,

    /// Open specified app.
    OpenApp(String),

    /// Basic system monitoring.
    StartBasicSystemMonitoring,

    /// If no other options are suitable, then this is a simple request from a language model.
    ///
    /// # Examples
    ///  - Can you help me with that?
    ///  - Please provide an answer.
    Answer,
}

struct AUsecases{
    inner: HashMap<String, Box<dyn Usecase + Send>>,
}

impl AUsecases {
    pub fn new() -> Self{
        let mut v: HashMap<String, Box<dyn Usecase + Send>> = HashMap::new();
        // Music stuff
        let music_status_usecase = GenericUsecase::new(|input: &str| music_control::get_music_status(input.into()));
        let play_next_usecase = GenericUsecase::new(|input|  music_control::play_next_track(input.into()));
        let play_prev_usecase = GenericUsecase::new(|input|  music_control::play_previous_track(input.into()));
        let play_resume_music =  GenericUsecase::new(|input|  music_control::play_or_resume_music(input.into()) );

        // Other stuff
        let app_opener = Box::new(OpenApp::default());
        let shutdown_usecase= GenericUsecase::new(|_| pc_mgmt::shutdown::shutdown());
        let reboot_usecase = GenericUsecase::new(|_| pc_mgmt::shutdown::reboot());
        let answer_usecase = GenericUsecase::new(|input| geranal_answer::answer(input.into()));
        let sysmon_usecase = GenericUsecase::new(|input| system_monitoring::start_basic_monitoring(input.into()));
        // Music stuff
        v.insert("music_status".into(),Box::new(music_status_usecase));
        v.insert("play_next".into(), Box::new(play_next_usecase));
        v.insert("play_prev".into(), Box::new(play_prev_usecase));
        v.insert("play_resume".into(), Box::new(play_resume_music));

        // Other stuff
        v.insert("open_app".into(),app_opener);
        v.insert("answer".into(), Box::new(answer_usecase));
        v.insert("sysmon".into(), Box::new(sysmon_usecase));
        v.insert("reboot".into(), Box::new(reboot_usecase));
        v.insert("shutdown".into(), Box::new(shutdown_usecase));
        Self { inner: v }
    }

    pub async fn execute_usecase(&self,usecase_id: &str,input: &str) -> Option<()>{
        let usecase = self.inner.get(usecase_id)?;
        if usecase.enabled(){
            return Some(usecase.execute(input.into()).await);
        }
        None
    }
}

impl Usecases {
    pub fn stringify_all() -> String {
        let strings = [Usecases::stringify_one()];
        let iter = strings.iter().map(|el| el.to_string() + "\n\n");
        String::from_iter(iter)
    }
    pub async fn execute(self, userinput: String) {
        let command = self;
        debug!("Dispatching command: {:?}", command);
        match command {
            Usecases::TurnOffMusic | Usecases::TurnOnMusic => {
                music_control::play_or_resume_music(userinput).await;
            }
            Usecases::GetMusicStatus => {
                music_control::get_music_status(userinput).await;
            }
            Usecases::PlayNextTrack => music_control::play_next_track(userinput).await,
            Usecases::PlayPrevTrack => music_control::play_previous_track(userinput).await,
            Usecases::StartBasicSystemMonitoring => {
                system_monitoring::start_basic_monitoring(userinput).await
            }
            Usecases::OpenApp(app) => open_app::open(app).await,
            Usecases::Answer => geranal_answer::answer(userinput).await,
            Usecases::Shutdown => do_dang(pc_mgmt::shutdown::shutdown).await,
            Usecases::Reboot => do_dang(pc_mgmt::shutdown::reboot).await,
        }
    }
}

async fn do_dang(h: impl std::ops::AsyncFnOnce()) {
    if CONFIG.usecases.test_dangerous_features {
        info!("Execute dangerous feature.");
        h().await
    } else {
        info!("Dangerous features doesn't execute.");
    }
}
