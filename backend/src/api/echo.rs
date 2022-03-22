use http::Error as HttpError;
use http::{Request, Response};
use hyper::body::{Body, HttpBody};
use std::pin::Pin;

pub async fn echo(req: Request<Body>) -> Result<Response<Body>, HttpError> {
    let (mut writer, responce_body) = Body::channel();

    tokio::spawn(async move {
        writer
            .send_data(format!("{} {} {:?}\r\n", req.method(), req.uri(), req.version()).into())
            .await
            .unwrap();
        for (key, value) in req.headers().iter() {
            writer
                .send_data(format!("{:?} {:?}\r\n", key, value).into())
                .await
                .unwrap();
        }
        writer.send_data("\r\n".into()).await.unwrap();
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
