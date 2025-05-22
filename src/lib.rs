use worker::*;
use console_error_panic_hook::set_once;

#[event(fetch)]
pub async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    set_once();

    let url = req.url()?;
    match url.path().as_ref() {
        "/kv-test" => {
            // Basic KV get/put demonstration
            let value = url.query().unwrap_or("default");
            let kv = env.kv("ArbEdgeKV")?;
            // Store value under a test key
            kv.put("test-key", value)?.execute().await?;
            // Retrieve stored value
            let retrieved = kv.get("test-key").await?.text().await?;
            Response::ok(retrieved.unwrap_or_default())
        }
        _ => Response::ok("Hello, ArbEdge in Rust!"),
    }
} 