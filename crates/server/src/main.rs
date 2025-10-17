use api::router;
use axum;

#[tokio::main]
async fn main() {
    let app = router::router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}