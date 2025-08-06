use ic_cdk::api::time;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static METRICS: RefCell<SystemMetrics> = RefCell::new(SystemMetrics::default());
}

#[derive(Debug, Default)]
pub struct SystemMetrics {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub histograms: HashMap<String, Vec<f64>>,
    pub last_updated: u64,
}

pub struct Metrics;

impl Metrics {
    pub fn increment_counter(name: &str) {
        Self::add_to_counter(name, 1);
    }
    
    pub fn add_to_counter(name: &str, value: u64) {
        let now = time();
        METRICS.with(|m| {
            let mut metrics = m.borrow_mut();
            *metrics.counters.entry(name.to_string()).or_insert(0) += value;
            metrics.last_updated = now;
        });
    }
    
    pub fn set_gauge(name: &str, value: f64) {
        let now = time();
        METRICS.with(|m| {
            let mut metrics = m.borrow_mut();
            metrics.gauges.insert(name.to_string(), value);
            metrics.last_updated = now;
        });
    }
    
    pub fn record_histogram(name: &str, value: f64) {
        let now = time();
        METRICS.with(|m| {
            let mut metrics = m.borrow_mut();
            let hist = metrics.histograms.entry(name.to_string()).or_insert_with(Vec::new);
            hist.push(value);
            
            // Keep only last 1000 values to prevent unbounded growth
            if hist.len() > 1000 {
                hist.remove(0);
            }
            
            metrics.last_updated = now;
        });
    }
    
    pub fn increment_inference_count() {
        Self::increment_counter("inferences_total");
    }
    
    pub fn record_inference_time(time_ms: u64) {
        Self::record_histogram("inference_time_ms", time_ms as f64);
    }
    
    pub fn increment_cache_hit() {
        Self::increment_counter("cache_hits_total");
    }
    
    pub fn increment_cache_miss() {
        Self::increment_counter("cache_misses_total");
    }
    
    pub fn record_tokens_generated(count: u32) {
        Self::add_to_counter("tokens_generated_total", count as u64);
    }
    
    pub fn get_counter(name: &str) -> u64 {
        METRICS.with(|m| {
            m.borrow().counters.get(name).copied().unwrap_or(0)
        })
    }
    
    pub fn get_gauge(name: &str) -> Option<f64> {
        METRICS.with(|m| {
            m.borrow().gauges.get(name).copied()
        })
    }
    
    pub fn get_histogram_stats(name: &str) -> Option<HistogramStats> {
        METRICS.with(|m| {
            let metrics = m.borrow();
            if let Some(values) = metrics.histograms.get(name) {
                if values.is_empty() {
                    return None;
                }
                
                let mut sorted = values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                
                let len = sorted.len();
                let sum: f64 = sorted.iter().sum();
                let mean = sum / len as f64;
                
                let p50 = sorted[len / 2];
                let p95 = sorted[(len as f64 * 0.95) as usize];
                let p99 = sorted[(len as f64 * 0.99) as usize];
                
                Some(HistogramStats {
                    count: len as u64,
                    sum,
                    mean,
                    min: sorted[0],
                    max: sorted[len - 1],
                    p50,
                    p95,
                    p99,
                })
            } else {
                None
            }
        })
    }
    
    pub fn get_all_metrics() -> serde_json::Value {
        METRICS.with(|m| {
            let metrics = m.borrow();
            serde_json::json!({
                "counters": metrics.counters,
                "gauges": metrics.gauges,
                "histogram_count": metrics.histograms.len(),
                "last_updated": metrics.last_updated
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct HistogramStats {
    pub count: u64,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}