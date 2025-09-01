const { chromium } = require('playwright');

(async () => {
    const browser = await chromium.launch({ headless: false });
    const page = await browser.newPage();
    
    // Load the debug HTML
    await page.goto(`file://${process.cwd()}/test-drag-debug.html`);
    
    // Try to drag card1 to col2
    const card1 = await page.locator('#card1');
    const col2 = await page.locator('#col2');
    
    console.log('Starting drag test...');
    
    // Method 1: hover and mouse events
    await card1.hover();
    await page.mouse.down();
    await col2.hover();
    await page.mouse.up();
    
    await page.waitForTimeout(1000);
    
    // Check if card moved
    const col2Cards = await col2.locator('.card').count();
    console.log('Cards in column 2:', col2Cards);
    
    if (col2Cards > 0) {
        console.log('✓ Drag successful!');
    } else {
        console.log('✗ Drag failed');
    }
    
    await page.waitForTimeout(3000);
    await browser.close();
})();