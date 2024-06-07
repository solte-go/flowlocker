pub mod error;

use error::Result;

use std::collections::HashMap;

use std::io::{BufRead, BufReader, Read};

use opentelemetry::global::BoxedTracer;
use opentelemetry::{
    global,
    trace::{Span, TraceContextExt, TraceError, Tracer},
    Context, KeyValue,
};
use opentelemetry_otlp;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::{trace, Resource};
use opentelemetry_semantic_conventions as semcov;

pub fn get_global_trace(tracer_name: String) -> BoxedTracer {
    global::tracer(tracer_name)
}

pub fn init_opentelemetry(tracer_name: String) -> Result<trace::Tracer> {
    // Helper function to read potentially available OneAgent data
    fn read_dt_metadata() -> Resource {
        fn read_single(path: &str, metadata: &mut Vec<KeyValue>) -> std::io::Result<()> {
            let mut file = std::fs::File::open(path)?;
            if path.starts_with("dt_metadata") {
                let mut name = String::new();
                file.read_to_string(&mut name)?;
                file = std::fs::File::open(name)?;
            }
            for line in BufReader::new(file).lines() {
                if let Some((k, v)) = line?.split_once('=') {
                    metadata.push(KeyValue::new(k.to_string(), v.to_string()))
                }
            }
            Ok(())
        }
        let mut metadata = Vec::new();
        for name in [
            "dt_metadata_e617c525669e072eebe3d0f08212e8f2.properties",
            "/var/lib/dynatrace/enrichment/dt_metadata.properties",
            "/var/lib/dynatrace/enrichment/dt_host_metadata.properties",
        ] {
            let _ = read_single(name, &mut metadata);
        }
        Resource::new(metadata)
    }

    // ===== GENERAL SETUP =====
    // let DT_API_TOKEN = env::var("DT_API_TOKEN").unwrap(); // TODO: change
    // let DT_API_URL = env::var("DT_API_URL").unwrap();

    // let mut map = HashMap::new();
    // map.insert("Authorization".to_string(), format!("Api-Token {}", DT_API_TOKEN));
    let mut resource = Resource::new([
        KeyValue::new(semcov::resource::SERVICE_NAME, tracer_name), //TODO Replace with the name of your application
    ]);
    resource = resource.merge(&read_dt_metadata());

    // ===== TRACING SETUP =====
    global::set_text_map_propagator(TraceContextPropagator::new());

    // let tracer = opentelemetry_jaeger::new_agent_pipeline()
    //     .with_trace_config(
    //         trace::config()
    //             .with_resource(resource)
    //             .with_sampler(trace::Sampler::AlwaysOn),
    //     )
    //     .install_batch(Tokio)?;

    //TODO Check HTTP exporter

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            trace::config()
                .with_resource(resource)
                .with_sampler(trace::Sampler::AlwaysOn),
        )
        .install_batch(runtime::Tokio)?;

    Ok(tracer)
}
