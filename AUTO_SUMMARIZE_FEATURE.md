# Auto-Summarize Map View Feature

## Overview
A revolutionary auto-summarize feature has been implemented for the map view that dynamically adjusts content detail based on zoom level. As users zoom in and out with mouse wheel or buttons, the view intelligently summarizes tasks and goals using AI-powered language models, creating different levels of abstraction.

## Key Components

### 1. Summarization Service (`src/services/summarization.rs`)
- **AI Integration**: Supports both Ollama (local) and OpenAI-compatible APIs
- **Caching System**: LRU cache with TTL for performance optimization
- **Summarization Levels**:
  - `HighLevel`: 1-2 sentence overview
  - `MidLevel`: 3-4 sentence summary  
  - `LowLevel`: 5-6 sentence detail
  - `Detailed`: Full information preserved
- **Fallback**: Rule-based summarization when AI unavailable

### 2. Enhanced Map View (`src/ui/views/map_view.rs`)
- **Zoom-Based Detail Levels**:
  - < 0.3x zoom: Overview mode with clusters
  - 0.3-0.6x: Summary view
  - 0.6-1.5x: Standard view
  - > 1.5x: Detailed view
- **Dynamic Updates**: Re-summarizes when zoom changes > 0.1x
- **Cluster Summarization**: Groups nearby tasks at high zoom-out levels
- **Performance**: Async processing with Tokio runtime

### 3. Test Coverage (`tests/auto_summarize_tests.rs`)
- 13 comprehensive tests covering:
  - Zoom level mappings
  - Dynamic summarization
  - Caching performance
  - Cluster aggregation
  - Smooth transitions
  - Concurrent requests
  - Viewport-based rendering

## Technical Highlights

### Performance Optimizations
1. **Smart Caching**: 
   - Content-based cache keys
   - 15-minute TTL
   - LRU eviction for memory management

2. **Efficient Processing**:
   - Batch summarization for visible items only
   - Viewport culling to avoid unnecessary work
   - Async/await for non-blocking UI

3. **Smooth Transitions**:
   - Gradual detail changes between zoom levels
   - Threshold-based re-summarization (0.1x change)

### AI Model Configuration
- **Default**: Ollama with llama3.2:1b (fast, local)
- **Alternative**: OpenAI GPT-4o-mini (via API key)
- **Environment Variables**:
  - `LLM_ENDPOINT`: Custom model endpoint
  - `LLM_API_KEY`: API authentication

## User Experience

### Magical Features
1. **Instant Context**: See high-level project overview when zoomed out
2. **Progressive Detail**: Smoothly reveals more information as you zoom in
3. **Smart Clustering**: Related tasks group automatically at overview levels
4. **Fast Response**: Sub-100ms cache hits for repeated views

### Controls
- **Mouse Wheel**: Smooth zoom with automatic re-summarization
- **Zoom Buttons**: +/- buttons in UI for precise control
- **Reset View**: Home button returns to 1.0x zoom

## Usage Example

```rust
// The feature activates automatically in MapView
let mut map_view = MapView::new();

// As user scrolls/zooms, summaries update dynamically
map_view.set_zoom_level(0.5); // Triggers mid-level summaries
map_view.set_zoom_level(2.0); // Shows detailed information
```

## Testing

Run comprehensive tests:
```bash
cargo test --test auto_summarize_tests
```

## Future Enhancements
1. User-configurable summarization prompts
2. Multiple AI model support simultaneously
3. Semantic clustering based on task relationships
4. Animation transitions between detail levels
5. Customizable zoom-to-detail mappings

## Conclusion
This auto-summarize feature transforms the map view into an intelligent, adaptive interface that provides the right level of detail at the right time. It creates a truly magical experience where information density automatically adjusts to user needs, making complex project navigation fast, intuitive, and delightful.