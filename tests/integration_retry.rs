use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use labelr::retry::retry_octocrab;
use octocrab::Octocrab;
use std::convert::Infallible;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[tokio::test]
async fn retry_handles_429_then_success() {
    // start a simple hyper server that returns 429 twice then 200
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    let make_svc = make_service_fn(move |_| {
        let c = c.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let c = c.clone();
                async move {
                    let path = req.uri().path().to_string();
                    if req.method() == Method::GET && path.ends_with("/labels") {
                        let prev = c.fetch_add(1, Ordering::SeqCst);
                        if prev < 2 {
                            let mut resp = Response::new(Body::from(""));
                            *resp.status_mut() = StatusCode::TOO_MANY_REQUESTS;
                            return Ok::<_, Infallible>(resp);
                        } else {
                            let body = "[]";
                            let resp = Response::new(Body::from(body));
                            return Ok::<_, Infallible>(resp);
                        }
                    }
                    Ok::<_, Infallible>(Response::new(Body::from("not found")))
                }
            }))
        }
    });

    let server = Server::bind(&([127, 0, 0, 1], 0).into()).serve(make_svc);
    let addr = server.local_addr();
    let server_handle = tokio::spawn(server);

    // build octocrab pointing to our server
    let base = format!("http://{}", addr);
    let octo = octocrab::OctocrabBuilder::default()
        .base_uri(&base)
        .unwrap()
        .personal_token("token")
        .build()
        .unwrap();

    // call retry_octocrab with octo.get to /repos/owner/repo/labels
    let route = format!("/repos/{}/{}/labels", "owner", "repo");

    let res: Result<Vec<octocrab::models::Label>, octocrab::Error> =
        retry_octocrab(|| octo.get(&route, None::<&()>), 5).await;
    assert!(res.is_ok());

    // shutdown server
    server_handle.abort();
}
