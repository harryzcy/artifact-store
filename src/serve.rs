use std::{convert::Infallible, io, net::SocketAddr};

use axum_core::{body::Body, extract::Request, response::Response};
use futures_util::{future::poll_fn, FutureExt};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
};
use tokio::net::{TcpListener, TcpStream};
use tower_hyper_http_body_compat::{HttpBody04ToHttpBody1, HttpBody1ToHttpBody04};
use tower_service::Service;

pub async fn serve<M, S>(tcp_listener: TcpListener, mut make_service: M) -> io::Result<()>
where
    M: for<'a> Service<IncomingStream<'a>, Error = Infallible, Response = S>,
    S: Service<Request, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send,
{
    loop {
        let (tcp_stream, remote_addr) = tcp_listener.accept().await?;
        let tcp_stream = TokioIo::new(tcp_stream);

        poll_fn(|cx| make_service.poll_ready(cx))
            .await
            .unwrap_or_else(|err| match err {});

        let service = make_service
            .call(IncomingStream {
                tcp_stream: &tcp_stream,
                remote_addr,
            })
            .await
            .unwrap_or_else(|err| match err {});

        let service = hyper1::service::service_fn(move |req: Request<hyper1::body::Incoming>| {
            let mut service = service.clone();

            match poll_fn(|cx| service.poll_ready(cx)).now_or_never() {
                Some(Ok(())) => {}
                Some(Err(err)) => match err {},
                None => {
                    // ...otherwise load shed
                    let mut res = Response::new(HttpBody04ToHttpBody1::new(Body::empty()));
                    *res.status_mut() = http::StatusCode::SERVICE_UNAVAILABLE;
                    return std::future::ready(Ok(res)).left_future();
                }
            }

            let future = service.call(req);

            async move {
                let response = future
                    .await
                    .unwrap_or_else(|err| match err {})
                    // wont need this when axum uses http-body 1.0
                    .map(HttpBody04ToHttpBody1::new);

                Ok::<_, Infallible>(response)
            }
            .right_future()
        });

        tokio::task::spawn(async move {
            match Builder::new(TokioExecutor::new())
                // upgrades needed for websockets
                .serve_connection_with_upgrades(tcp_stream, service)
                .await
            {
                Ok(()) => {}
                Err(_err) => {
                    // This error only appears when the client doesn't send a request and
                    // terminate the connection.
                    //
                    // If client sends one request then terminate connection whenever, it doesn't
                    // appear.
                }
            }
        });
    }
}

/// An incoming stream.
///
/// Used with [`serve`] and [`IntoMakeServiceWithConnectInfo`].
///
/// [`IntoMakeServiceWithConnectInfo`]: crate::extract::connect_info::IntoMakeServiceWithConnectInfo
#[derive(Debug)]
pub struct IncomingStream<'a> {
    tcp_stream: &'a TokioIo<TcpStream>,
    remote_addr: SocketAddr,
}

impl IncomingStream<'_> {
    /// Returns the local address that this stream is bound to.
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.tcp_stream.inner().local_addr()
    }

    /// Returns the remote address that this stream is bound to.
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }
}
