use std::alloc::{GlobalAlloc, System, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};

struct AllocCounter;

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for AllocCounter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        System.alloc(layout)
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: AllocCounter = AllocCounter;

fn main() {
    println!("Testing allocations in map view...");
    
    // Run a simple egui app and monitor allocations
    let options = eframe::NativeOptions::default();
    
    let _ = eframe::run_native(
        "Alloc Test",
        options,
        Box::new(|cc| {
            Box::new(AllocTestApp::new(cc))
        }),
    );
}

struct AllocTestApp {
    app: plon::ui::PlonApp,
    last_alloc: usize,
    last_dealloc: usize,
}

impl AllocTestApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let repository = runtime.block_on(async {
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .connect("sqlite::memory:")
                .await
                .unwrap();
            
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .unwrap();
            
            plon::repository::Repository::new(pool)
        });
        
        Self {
            app: plon::ui::PlonApp::new(cc, repository),
            last_alloc: 0,
            last_dealloc: 0,
        }
    }
}

impl eframe::App for AllocTestApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        let allocs = ALLOC_COUNT.load(Ordering::Relaxed);
        let deallocs = DEALLOC_COUNT.load(Ordering::Relaxed);
        
        let new_allocs = allocs - self.last_alloc;
        let new_deallocs = deallocs - self.last_dealloc;
        
        if new_allocs > 10000 || new_deallocs > 10000 {
            println!("⚠️ High allocation rate: +{} allocs, +{} deallocs", new_allocs, new_deallocs);
        }
        
        self.last_alloc = allocs;
        self.last_dealloc = deallocs;
        
        // Simulate scroll events
        ctx.input_mut(|i| {
            i.events.push(eframe::egui::Event::Scroll(eframe::egui::vec2(5.0, 10.0)));
        });
        
        self.app.update(ctx, frame);
        ctx.request_repaint();
    }
}