use crate::CoreResult;

pub trait RpcServer: Send + Sync + fmt::Debug {
    fn start(&self) -> CoreResult<()>;
    fn stop(&self) -> CoreResult<()>;
    fn register_method(&mut self, name: &str, handler: RpcMethod) -> CoreResult<()>;
}

pub type RpcMethod = fn(params: serde_json::Value) -> CoreResult<serde_json::Value>;

pub trait RestApi: Send + Sync + fmt::Debug {
    fn route(&self, path: &str, handler: RestHandler) -> CoreResult<()>;
    fn start(&self, addr: std::net::SocketAddr) -> CoreResult<()>;
}

pub type RestHandler = fn(request: RestRequest) -> CoreResult<RestResponse>;

#[derive(Clone, Debug)]
pub struct RestRequest {
    pub method: http::Method,
    pub path: String,
    pub headers: http::HeaderMap,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct RestResponse {
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
    pub body: Vec<u8>,
}

pub trait GrpcService: Send + Sync + fmt::Debug {
    fn start(&self, addr: std::net::SocketAddr) -> CoreResult<()>;
    fn stop(&self) -> CoreResult<()>;
}
