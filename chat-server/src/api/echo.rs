use bytes::Bytes;
use http::Error as HttpError;
use http::{Request, Response};
use hyper::body::{Body, HttpBody};
use std::error::Error as StdError;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};

use log::info;

type AsyncError = Box<dyn StdError + Send + Sync + 'static>;
type AsyncResult = Result<bytes::Bytes, AsyncError>;
type IntoBody = Box<(dyn Stream<Item = AsyncResult> + Send + 'static)>;

pub async fn echo(req: Request<Body>) -> Result<Response<Body>, HttpError> {
    // let (mut writer, responce_body) = Body::channel();
    let (writer, response_stream) = mpsc::channel::<AsyncResult>(16);
    let response_body: IntoBody = Box::new(ReceiverStream::new(response_stream));
    let response = Body::from(response_body);

    info!("response is {:?}", response);

    tokio::spawn(async move {
        let first_line = format!("{} {} {:?}\r\n", req.method(), req.uri(), req.version());
        writer.send(Ok(first_line.into())).await.unwrap();

        for (key, value) in req.headers().iter() {
            writer
                .send(Ok(format!("{:?} {:?}\r\n", key, value).into()))
                .await
                .unwrap();
        }
        writer
            .send(Ok(Bytes::from(b"\r\n" as &'static [u8])))
            .await
            .unwrap();

        let mut body = Box::pin(req.into_body());
        info!("request body is {:?}", body);

        while let Some(Ok(data)) = body.next().await {
            writer.send(Ok(data)).await.unwrap();
        }
    });

    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .body(response)
}
