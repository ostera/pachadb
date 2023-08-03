extern crate console_error_panic_hook;
use log::*;
use pachadb_core::*;
use pachadb_nanolog::*;
use std::panic;
use worker::*;

#[event(queue)]
async fn handle_event(batch: MessageBatch<Uri>, env: Env, _ctx: Context) -> Result<()> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());

    Ok(())
}

#[event(fetch)]
async fn handle_request(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());

    let router = Router::new();

    router
        .post_async("/", |mut req, ctx| async move {
            let query_req: QueryReq = req.json().await?;
            info!("Executing {:?}", query_req);
            Response::ok(query_req.query)
        })
        .run(req, env)
        .await
}
