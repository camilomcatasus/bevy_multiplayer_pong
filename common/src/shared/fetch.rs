use http_types::{Request, Response};
use smol::Async;
use anyhow::{Context, bail, Result, Error};
use std::{net::{TcpStream, ToSocketAddrs}};
use bevy::{prelude::*};

pub async fn fetch(req: Request) -> Result<Response> {
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
