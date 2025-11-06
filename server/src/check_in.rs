use std::{net::{TcpStream, ToSocketAddrs}, str::FromStr, time::Duration};
use http_types::{Request, Response};
use bevy::{prelude::*, tasks::IoTaskPool};
use smol::{prelude::*, Async};
use anyhow::{Context, bail, Result, Error};

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

async fn fetch(req: Request) -> Result<Response> {
    // Figure out the host and the port.
    let host = req.url().host().context("cannot parse host")?.to_string();
    let port = req
        .url()
        .port_or_known_default()
        .context("cannot guess port")?;

    // Connect to the host.
    let socket_addr = {
        let host = host.clone();
        smol::unblock(move || (host.as_str(), port).to_socket_addrs())
            .await?
            .next()
            .context("cannot resolve address")?
    };
    let stream = Async::<TcpStream>::connect(socket_addr).await?;

    // Send the request and wait for the response.
    let resp = match req.url().scheme() {
        "http" => async_h1::connect(stream, req).await.map_err(Error::msg)?,
        "https" => {
            // In case of HTTPS, establish a secure TLS connection first.
            let stream = async_native_tls::connect(&host, stream).await?;
            async_h1::connect(stream, req).await.map_err(Error::msg)?
        }
        scheme => bail!("unsupported scheme: {}", scheme),
    };
    Ok(resp)
}
