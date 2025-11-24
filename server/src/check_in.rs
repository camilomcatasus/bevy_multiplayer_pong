use std::{str::FromStr, time::Duration};
use common::shared::fetch::fetch;
use http_types::{Request, Response};
use bevy::{prelude::*, tasks::IoTaskPool};

use crate::{Args, LobbyInfo};
pub struct CheckInPlugin;


#[derive(Resource)]
struct CheckInData {
    timer: Timer,
}


fn check_in(
    time: Res<Time>,
    lobby_info: Res<LobbyInfo>,
    args: Res<Args>,
    mut check_in_data: ResMut<CheckInData>,
) {
    check_in_data.timer.tick(time.delta());

    if check_in_data.timer.is_finished() {
        let lobby_info_clone = lobby_info.clone();

        let url = http_types::Url::from_str(&format!("http://127.0.0.1:{}/checkin", args.broker_port))
            .expect("Cannot parse check-in url");
        let body = http_types::Body::from_json(&lobby_info_clone)
            .expect("Cannot prase json body from lobby_info");
        let mut request = Request::post(url);
        request.set_body(body);
        
        let check_in_task = IoTaskPool::get().spawn(async move {
            let res = fetch(request).await;
            if let Err(err) = res {
                log::error!("Could not send request. Err({err:?})");
            }
        });

        check_in_task.detach();
    }
}

impl Plugin for CheckInPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .insert_resource(CheckInData {
                timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
            })
            .add_systems(Update, check_in);
    }
}
