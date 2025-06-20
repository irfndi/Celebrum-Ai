use worker::{Request, Response, Result};

/// Handle CORS preflight requests
pub fn handle_cors_preflight(req: &Request) -> Result<Request> {
    // For now, just return the request as-is
    // In a production environment, you would implement proper CORS handling
    req.clone()
}

/// Add CORS headers to response
pub fn add_cors_headers(mut response: Response) -> Result<Response> {
    let headers = response.headers_mut();

    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS",
    )?;
    headers.set(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization",
    )?;
    headers.set("Access-Control-Max-Age", "86400")?;

    Ok(response)
}
