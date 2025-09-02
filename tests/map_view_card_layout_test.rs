#[cfg(test)]
mod map_view_card_layout_tests {
    use std::fs;
    
    #[test]
    fn test_diagnose_map_view_card_issues() {
        println!("\n=== MAP VIEW CARD LAYOUT DIAGNOSIS ===\n");
        
        // Check if map view files exist
        let map_files = vec![
            "src/ui_dioxus/views/map_view_simple.rs",
            "src/ui_dioxus/views/map_final.rs",
            "src/ui_dioxus/views/mod.rs",
        ];
        
        println!("Checking map view files:");
        for file in &map_files {
            if std::path::Path::new(file).exists() {
                println!("  ✓ {} exists", file);
            } else {
                println!("  ✗ {} not found", file);
            }
        }
        
        // Read the mod.rs to see which map view is being used
        let mod_content = fs::read_to_string("src/ui_dioxus/views/mod.rs")
            .expect("Could not read mod.rs");
        
        println!("\nActive map view modules:");
        if mod_content.contains("pub mod map_view_simple") && !mod_content.contains("// pub mod map_view_simple") {
            println!("  ✓ map_view_simple is active");
        }
        if mod_content.contains("pub mod map_final") && !mod_content.contains("// pub mod map_final") {
            println!("  ✓ map_final is active");
        }
    }
    
    #[test]
    fn test_analyze_map_card_structure() {
        println!("\n=== ANALYZING MAP CARD STRUCTURE ===\n");
        
        // Read map_final.rs since it's likely the active one
        if let Ok(content) = fs::read_to_string("src/ui_dioxus/views/map_final.rs") {
            // Look for card styling
            let lines: Vec<&str> = content.lines().collect();
            
            println!("Searching for card-related styles:");
            let mut found_issues = Vec::new();
            
            for (i, line) in lines.iter().enumerate() {
                // Look for potential layout issues
                if line.contains("padding:") && line.contains("px") {
                    let padding = extract_padding(line);
                    if let Some(p) = padding {
                        if p < 10 {
                            found_issues.push(format!("Line {}: Small padding ({}px)", i+1, p));
                        }
                    }
                }
                
                if line.contains("width:") && line.contains("px") {
                    let width = extract_size(line, "width:");
                    if let Some(w) = width {
                        if w < 200 {
                            found_issues.push(format!("Line {}: Small width ({}px)", i+1, w));
                        }
                    }
                }
                
                if line.contains("height:") && line.contains("px") {
                    let height = extract_size(line, "height:");
                    if let Some(h) = height {
                        if h < 80 {
                            found_issues.push(format!("Line {}: Small height ({}px)", i+1, h));
                        }
                    }
                }
            }
            
            if !found_issues.is_empty() {
                println!("\n⚠️ Potential issues found:");
                for issue in found_issues {
                    println!("  - {}", issue);
                }
            }
            
            // Check for button/control overlap patterns
            println!("\nChecking for common overlap patterns:");
            if content.contains("position: absolute") && content.contains("button") {
                println!("  ⚠️ Found absolute positioning with buttons - potential overlap");
            }
            if content.contains("z-index") {
                println!("  ⚠️ Found z-index usage - may cause stacking issues");
            }
            if !content.contains("flex-direction") && !content.contains("display: flex") {
                println!("  ⚠️ No flexbox layout found - rigid positioning likely");
            }
        }
    }
    
    fn extract_padding(line: &str) -> Option<i32> {
        if let Some(start) = line.find("padding:") {
            let after = &line[start + 8..];
            if let Some(px_pos) = after.find("px") {
                let num_str = &after[..px_pos].trim();
                if let Ok(num) = num_str.parse::<i32>() {
                    return Some(num);
                }
            }
        }
        None
    }
    
    fn extract_size(line: &str, pattern: &str) -> Option<i32> {
        if let Some(start) = line.find(pattern) {
            let after = &line[start + pattern.len()..];
            if let Some(px_pos) = after.find("px") {
                let num_str = &after[..px_pos].trim();
                if let Ok(num) = num_str.parse::<i32>() {
                    return Some(num);
                }
            }
        }
        None
    }
    
    #[test]
    fn test_expected_card_layout() {
        println!("\n=== EXPECTED MAP CARD LAYOUT ===\n");
        
        println!("Proper card structure should have:");
        println!("  1. Adequate padding (16-20px minimum)");
        println!("  2. Minimum width (250px for readability)");
        println!("  3. Proper button spacing (8-12px gaps)");
        println!("  4. No overlapping elements");
        println!("  5. Clear visual hierarchy");
        println!();
        println!("Common fixes needed:");
        println!("  • Increase card size");
        println!("  • Use flexbox for button layout");
        println!("  • Add proper spacing between elements");
        println!("  • Ensure clickable areas don't overlap");
    }
}