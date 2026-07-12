use tracing::{Level, Subscriber};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Layer, Registry};

pub trait Logger: Send + Sync + fmt::Debug {
    fn init(&self) -> CoreResult<()>;
    fn shutdown(&self) -> CoreResult<()>;
}

pub struct LoggerBuilder {
    pub filter: EnvFilter,
    pub format: LogFormat,
    pub output: LogOutput,
}

#[derive(Clone, Debug)]
pub enum LogFormat {
    Text,
    Json,
}

#[derive(Clone, Debug)]
pub enum LogOutput {
    Stdout,
    Stderr,
    File(std::path::PathBuf),
}

impl LoggerBuilder {
    pub fn new() -> Self {
        Self {
            filter: EnvFilter::from_default_env().add_directive(Level::INFO.into()),
            format: LogFormat::Text,
            output: LogOutput::Stdout,
        }
    }

    pub fn with_filter(mut self, filter: EnvFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    pub fn build(self) -> CoreResult<impl Subscriber + Send + Sync> {
        let subscriber = Registry::default().with(self.filter);

        let layer: Box<dyn Layer<Registry> + Send + Sync> = match self.format {
            LogFormat::Text => Box::new(tracing_subscriber::fmt::layer().with_target(true).boxed()),
            LogFormat::Json => Box::new(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(true)
                    .boxed(),
            ),
        };

        Ok(subscriber.with(layer))
    }
}
