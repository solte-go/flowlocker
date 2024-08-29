use std::thread;
use std::time::Duration;

use metrics::gauge;
use peak_alloc::PeakAlloc;
use sysinfo::{System, Disks};

#[derive(Debug)]
pub struct MetricsCollector {
    sys_collector: System,
    disk_collecor: Disks,
    app_collector: PeakAlloc,
}

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

impl Default for MetricsCollector {
    fn default() -> Self {
        let sys = System::new_all();
        let mut disks = Disks::new();
        let app: PeakAlloc = PEAK_ALLOC;

        MetricsCollector { sys_collector: sys, app_collector: app, disk_collecor: disks }
    }
}

impl MetricsCollector {
    fn memory_usage(&self) {
        const MEM_USAGE: &str = "sys_stat";
        const APP_MEMORY_USAGE: &str = "memory_usage_in_mb";
        let memory_used = self.sys_collector.used_memory();
        gauge!("memory_usage", mb(memory_used as f64), MEM_USAGE => APP_MEMORY_USAGE);
    }

    fn total_memory(&self) {
        const MEM_TOTAL: &str = "sys_stat";
        const APP_MEM_TOTAL: &str = "memory_total_in_mb";
        let memory_used = self.sys_collector.total_memory();
        gauge!("memory_usage", mb(memory_used as f64), MEM_TOTAL => APP_MEM_TOTAL);
    }

    fn cpu_load(&mut self) {
        self.sys_collector.refresh_cpu(); // Refreshing CPU information.
        for (i, cpu) in self.sys_collector.cpus().iter().enumerate() {
            let cpu_label: &str = "sys_stat";
            let cpu_info = format!("cpu_{}_usage", i);
            gauge!("cpu_usage".to_string(), cpu.cpu_usage() as f64, cpu_label => cpu_info);
        }
    }

    fn disk_util(&mut self) {
        self.disk_collecor.refresh_list();
        for disk in &self.disk_collecor {
            let disk_label:&str = "sys_stat";
            let disk_info = format!("{:?}", disk.name());
            gauge!("disk_available_space", mb(disk.available_space() as f64), disk_label => disk_info);

            let disk_label:&str = "sys_stat";
            let disk_info = format!("{:?}", disk.name());
            gauge!("disk_total_space", mb(disk.total_space() as f64), disk_label => disk_info);
        }
    }

    fn app_mem_alloc(&self) {
        const INSTANCE_MEM_TOTAL: &str = "app_stat";
        const LABEL_TOTAL: &str = "app_memory_total_in_mb";
        let peak_memory_used = self.app_collector.peak_usage_as_mb();
        gauge!("app_memory_usage", peak_memory_used as f64, INSTANCE_MEM_TOTAL => LABEL_TOTAL);

        const INSTANCE_MEM_CURRENT: &str = "app_stat";
        const LABEL_CURRENT: &str = "app_currently_memory_in_mb";
        let current_memory_used = self.app_collector.current_usage_as_mb();
        gauge!("app_memory_usage", current_memory_used as f64, INSTANCE_MEM_CURRENT => LABEL_CURRENT);
    }
}

fn mb(x: f64) -> f64 { x / (1024.0 * 1024.0) }
fn gb(x: f64) -> f64 { x / (1024.0 * 1024.0 * 1024.0) }

pub fn register_metrics(mut mc: MetricsCollector) {
    loop {
        mc.sys_collector.refresh_all();

        mc.memory_usage();
        mc.total_memory();
        mc.cpu_load();
        mc.disk_util();
        mc.app_mem_alloc();
        thread::sleep(Duration::from_secs(1));
    }
}