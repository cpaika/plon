const { chromium } = require('playwright');

(async () => {
    const browser = await chromium.launch({ headless: false });
    const page = await browser.newPage();
    
    // Load the kanban HTML
    await page.goto(`file://${process.cwd()}/test-kanban.html`);
    
    const todoCard = page.locator('.kanban-card').first();
    
    // Check initial opacity
    const initialOpacity = await todoCard.evaluate(el => 
      window.getComputedStyle(el).opacity
    );
    console.log('Initial opacity:', initialOpacity);
    
    // Start dragging
    console.log('Starting drag...');
    await todoCard.hover();
    await page.mouse.down();
    
    // Move mouse to trigger drag
    await page.mouse.move(200, 200);
    await page.waitForTimeout(100);
    
    // Check opacity during drag
    const dragOpacity = await todoCard.evaluate(el => 
      window.getComputedStyle(el).opacity
    );
    console.log('Drag opacity:', dragOpacity);
    
    // Check inline style
    const inlineOpacity = await todoCard.evaluate(el => el.style.opacity);
    console.log('Inline opacity:', inlineOpacity);
    
    // Check if dragging class is added
    const hasDragging = await todoCard.evaluate(el => el.classList.contains('dragging'));
    console.log('Has dragging class:', hasDragging);
    
    await page.waitForTimeout(2000);
    await page.mouse.up();
    await browser.close();
})();