use std::time::Instant;

pub struct FrameMetrics {
    pub frame_count: u64,
    pub event_count: u64,
    pub scroll_events: u64,
    pub pan_events: u64,
    pub last_frame: Instant,
    pub slow_frames: Vec<(u64, std::time::Duration)>,
    pub event_processing_times: Vec<std::time::Duration>,
    pub render_times: Vec<std::time::Duration>,
}

impl FrameMetrics {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            event_count: 0,
            scroll_events: 0,
            pan_events: 0,
            last_frame: Instant::now(),
            slow_frames: Vec::new(),
            event_processing_times: Vec::new(),
            render_times: Vec::new(),
        }
    }
    
    pub fn start_frame(&mut self) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame);
        
        if frame_time > std::time::Duration::from_millis(50) {
            println!("⚠️ SLOW FRAME {}: {:?}", self.frame_count, frame_time);
            self.slow_frames.push((self.frame_count, frame_time));
        }
        
        self.frame_count += 1;
        self.last_frame = now;
        
        if self.frame_count % 100 == 0 {
            self.print_stats();
        }
    }
    
    pub fn record_event(&mut self, event_type: &str) {
        self.event_count += 1;
        match event_type {
            "scroll" => self.scroll_events += 1,
            "pan" => self.pan_events += 1,
            _ => {}
        }
        
        if self.event_count % 1000 == 0 {
            println!("📊 Events processed: {} (scroll: {}, pan: {})", 
                     self.event_count, self.scroll_events, self.pan_events);
        }
    }
    
    pub fn print_stats(&self) {
        println!("\n=== Frame {} Stats ===", self.frame_count);
        println!("Total events: {}", self.event_count);
        println!("Scroll events: {}", self.scroll_events);
        println!("Pan events: {}", self.pan_events);
        println!("Slow frames: {}", self.slow_frames.len());
        
        if !self.event_processing_times.is_empty() {
            let avg = self.event_processing_times.iter().sum::<std::time::Duration>() 
                / self.event_processing_times.len() as u32;
            println!("Avg event processing: {:?}", avg);
        }
    }
}