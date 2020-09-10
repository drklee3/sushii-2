use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use prometheus::{Encoder, TextEncoder};
use std::sync::Arc;

use crate::model::{SushiiConfig, Metrics};

async fn serve_req(_req: Request<Body>, metrics: Arc<Metrics>) -> Result<Response<Body>, hyper::Error> {
    let encoder = TextEncoder::new();

    let mut buffer = vec![];
    let metric_families = metrics.registry.gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .unwrap();

    Ok(response)
}

pub async fn start(conf: Arc<SushiiConfig>, metrics: Arc<Metrics>) {
    let addr = (conf.metrics_interface, conf.metrics_port).into();
    tracing::info!("Metrics server listening on http://{}", addr);

    let serve_future = Server::bind(&addr).serve(make_service_fn(move |_| {
        let metrics = metrics.clone();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| serve_req(req, metrics.clone())))
        }
    }));

    if let Err(err) = serve_future.await {
        tracing::warn!("Metrics server error: {}", err);
    }
}
