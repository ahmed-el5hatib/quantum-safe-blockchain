use prometheus::{Counter, Gauge, Histogram, Registry};

pub trait Metrics: Send + Sync + fmt::Debug {
    fn registry(&self) -> &Registry;
    fn register_counter(&self, name: &str, help: &str) -> CoreResult<Counter>;
    fn register_gauge(&self, name: &str, help: &str) -> CoreResult<Gauge>;
    fn register_histogram(
        &self,
        name: &str,
        help: &str,
        buckets: Option<Vec<f64>>,
    ) -> CoreResult<Histogram>;
}

pub trait Exporter: Send + Sync + fmt::Debug {
    fn export(&self) -> CoreResult<Vec<u8>>;
    fn start(&self, interval: std::time::Duration) -> CoreResult<()>;
    fn stop(&self) -> CoreResult<()>;
}
