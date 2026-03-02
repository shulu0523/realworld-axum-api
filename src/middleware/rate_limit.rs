use axum::body::Body;
use axum::http::{Request, Response};
use axum::middleware::Next;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

// 速率限制配置
const MAX_REQUESTS: u32 = 10; // 最大请求数
const WINDOW_SIZE: Duration = Duration::from_secs(60); // 时间窗口（60秒）

// 存储每个IP的请求信息
#[derive(Debug)]
struct RateLimitInfo {
    requests: Vec<Instant>,
}

// 全局存储，使用 Lazy 和 Mutex 确保线程安全
static RATE_LIMIT_STORE: Lazy<Mutex<HashMap<String, RateLimitInfo>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// 速率限制中间件
pub async fn rate_limit_middleware(req: Request<Body>, next: Next) -> Response<Body> {
    // 获取客户端IP
    let ip = if let Some(forwarded_for) = req.headers().get("X-Forwarded-For") {
        if let Ok(ip_str) = forwarded_for.to_str() {
            ip_str.to_string()
        } else {
            "unknown".to_string()
        }
    } else if let Some(connect_info) = req.extensions().get::<axum::extract::ConnectInfo<std::net::SocketAddr>>() {
        connect_info.ip().to_string()
    } else {
        "unknown".to_string()
    };

    // 检查速率限制
    {
        let mut store = RATE_LIMIT_STORE.lock().unwrap();
        let now = Instant::now();

        // 获取或创建IP的速率限制信息
        let rate_limit_info = store.entry(ip.to_string()).or_insert(RateLimitInfo {
            requests: Vec::new(),
        });

        // 清理过期的请求
        rate_limit_info
            .requests
            .retain(|&time| now.duration_since(time) < WINDOW_SIZE);

        // 检查是否超过限制
        if rate_limit_info.requests.len() >= MAX_REQUESTS as usize {
            return Response::builder()
                .status(429)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error":"Too many requests"}"#))
                .unwrap();
        }

        // 记录新请求
        rate_limit_info.requests.push(now);
    }

    // 继续处理请求
    next.run(req).await
}
