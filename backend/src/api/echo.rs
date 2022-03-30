use bytes::BytesMut;
use http::Error as HttpError;
use http::{Request, Response};
use hyper::body::{Body, HttpBody};
use std::pin::Pin;

pub async fn echo(req: Request<Body>) -> Result<Response<Body>, HttpError> {
    let (mut writer, responce_body) = Body::channel();

    tokio::spawn(async move {
        let mut heads = BytesMut::with_capacity(2 * 1024);
        heads.extend_from_slice(
            format!("{} {} {:?}\r\n", req.method(), req.uri(), req.version()).as_bytes(),
        );
        for (key, value) in req.headers().iter() {
            heads.extend_from_slice(format!("{:?} {:?}\r\n", key, value).as_bytes());
        }
        heads.extend_from_slice(b"\r\n");

        // TODO: figure out how to send the body and header in one packet, if possible

        writer.send_data(heads.freeze()).await.unwrap();
        let mut body = req.into_body();
        let mut pinned_body = Pin::new(&mut body);
        while let Some(Ok(data)) = pinned_body.data().await {
            writer.send_data(data).await.unwrap();
        }
    });

    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .body(responce_body)
}
