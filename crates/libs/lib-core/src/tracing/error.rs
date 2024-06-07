use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    CantInitTracer(String),
    #[from]
    TracingInitFail(opentelemetry::trace::TraceError),
}
