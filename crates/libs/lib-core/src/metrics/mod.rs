use axum::middleware::Next;
use axum::{body::Body, extract::Request, http::StatusCode, response::Response};
use lazy_static::lazy_static;
use peak_alloc::PeakAlloc;
use prometheus::{Gauge, GaugeVec, HistogramOpts, HistogramVec, IntCounter, Registry};
use regex::Regex;
use regex_macro::regex;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use sysinfo::{Disks, System};

use std::thread;
use std::time::Duration;

lazy_static! {
    static ref UUID_REGEX: Regex =
        Regex::new(r"/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap();
}

#[derive(Clone)]
pub struct MetricsMiddleware {
    requests_total: IntCounter,
    request_duration: HistogramVec,
}

impl MetricsMiddleware {
    pub fn new(requests_total: IntCounter, request_duration: HistogramVec) -> Self {
        Self {
            requests_total,
            request_duration,
        }
    }
}

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref HTTP_REQUESTS_TOTAL: IntCounter =
        IntCounter::new("http_requests_total", "Total number of HTTP requests")
            .expect("metric can be created");
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds"
        ),
        &["method", "path", "status"]
    )
    .expect("metric can be created");

    // New metrics
       pub static ref MEMORY_USAGE: Gauge = Gauge::new("memory_usage_mb", "Memory usage in MB").expect("metric can be created");
       pub static ref MEMORY_TOTAL: Gauge = Gauge::new("memory_total_mb", "Total memory in MB").expect("metric can be created");
       pub static ref CPU_USAGE: GaugeVec = GaugeVec::new(
              prometheus::opts!("cpu_usage_percent", "CPU usage percentage"),
              &["cpu"]
          ).expect("metric can be created");
          pub static ref DISK_AVAILABLE: GaugeVec = GaugeVec::new(
              prometheus::opts!("disk_available_mb", "Available disk space in MB"),
              &["disk"]
          ).expect("metric can be created");
          pub static ref DISK_TOTAL: GaugeVec = GaugeVec::new(
              prometheus::opts!("disk_total_mb", "Total disk space in MB"),
              &["disk"]
          ).expect("metric can be created");
       pub static ref APP_MEMORY_USAGE: Gauge = Gauge::new("app_memory_usage_mb", "Application memory usage in MB").expect("metric can be created");
}

#[global_allocator]
pub static PEAK_ALLOC: PeakAlloc = PeakAlloc;

pub fn register_metrics() {
    REGISTRY
        .register(Box::new(HTTP_REQUESTS_TOTAL.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(HTTP_REQUEST_DURATION.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(MEMORY_USAGE.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(MEMORY_TOTAL.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(CPU_USAGE.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(DISK_AVAILABLE.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(DISK_TOTAL.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(APP_MEMORY_USAGE.clone()))
        .expect("collector can be registered");
    let mut sys = System::new_all();
    let mut disks = Disks::new();

    thread::spawn(move || {
        loop {
            sys.refresh_all();
            disks.refresh_list();

            // Memory usage
            MEMORY_USAGE.set(mb(sys.used_memory() as f64));
            MEMORY_TOTAL.set(mb(sys.total_memory() as f64));

            // CPU usage
            for (i, cpu) in sys.cpus().iter().enumerate() {
                CPU_USAGE
                    .with_label_values(&[&format!("cpu_{}", i)])
                    .set(cpu.cpu_usage() as f64);
            }

            // Disk usage
            for disk in &disks {
                let name = disk.name().to_str().unwrap_or("unknown");
                DISK_AVAILABLE
                    .with_label_values(&[name])
                    .set(mb(disk.available_space() as f64));
                DISK_TOTAL
                    .with_label_values(&[name])
                    .set(mb(disk.total_space() as f64));
            }

            // Application memory usage
            APP_MEMORY_USAGE.set(PEAK_ALLOC.peak_usage_as_mb() as f64);

            thread::sleep(Duration::from_secs(1));
        }
    });
}

fn mb(x: f64) -> f64 {
    x / (1024.0 * 1024.0)
}

pub async fn track_metrics(
    middleware: Arc<MetricsMiddleware>,
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let original_path = req.uri().path().to_owned();

    // TODO think about efficiancy
    let path = UUID_REGEX.replace_all(&original_path, "/:id").into_owned();

    // Increment the total requests counter
    middleware.requests_total.inc();

    let response = next.run(req).await;

    // Record the request duration
    let duration = start.elapsed().as_secs_f64();
    middleware
        .request_duration
        .with_label_values(&[method.as_str(), &path, response.status().as_str()])
        .observe(duration);

    response
}

pub async fn metrics_handler() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    encoder.encode(&REGISTRY.gather(), &mut buffer).unwrap();

    String::from_utf8(buffer).unwrap()
}
