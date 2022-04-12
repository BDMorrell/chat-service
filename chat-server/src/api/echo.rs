use bytes::Bytes;
use http::Error as HttpError;
use http::{HeaderMap, Request, Response};
use hyper::body::Body;
use std::error::Error as StdError;
use std::pin::Pin;
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};

type AsyncError = Box<dyn StdError + Send + Sync + 'static>;
type AsyncResult = Result<bytes::Bytes, AsyncError>;
type IntoBody = Box<(dyn Stream<Item = AsyncResult> + Send + 'static)>;

pub async fn echo(req: Request<Body>) -> Result<Response<Body>, HttpError> {
    let (writer, response) = make_streaming_body(16);

    tokio::spawn(async move {
        let first_line = format!("{} {} {:?}\r\n", req.method(), req.uri(), req.version());
        writer.send(Ok(first_line.into())).await.unwrap();

        echo_headers(req.headers(), &writer).await;

        echo_body(Box::pin(req.into_body()), &writer).await;
    });

    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .body(response)
}

fn make_streaming_body(buffer: usize) -> (Sender<AsyncResult>, Body) {
    let (writer, response_stream) = mpsc::channel::<AsyncResult>(buffer);
    let response_body: IntoBody = Box::new(ReceiverStream::new(response_stream));
    let response = Body::from(response_body);
    (writer, response)
}

async fn echo_headers(headers: &HeaderMap, destination: &Sender<AsyncResult>) {
    for (key, value) in headers.iter() {
        destination
            .send(Ok(format!("{:?} {:?}\r\n", key, value).into()))
            .await
            .unwrap();
    }
    destination
        .send(Ok(Bytes::from(b"\r\n" as &'static [u8])))
        .await
        .unwrap();
}

async fn echo_body(mut input_body: Pin<Box<Body>>, destination: &Sender<AsyncResult>) {
    while let Some(Ok(data)) = input_body.next().await {
        destination.send(Ok(data)).await.unwrap();
    }
}
