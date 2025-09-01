const { chromium } = require('playwright');

(async () => {
    const browser = await chromium.launch({ headless: false });
    const page = await browser.newPage();
    
    // Load the kanban HTML
    await page.goto(`file://${process.cwd()}/test-kanban.html`);
    
    const todoCard = page.locator('.kanban-card').first();
    const doneColumn = page.locator('.kanban-column[data-status="done"]').first();
    
    // Start dragging
    console.log('Starting drag...');
    await todoCard.hover();
    await page.mouse.down();
    
    // Get initial background
    const initialBg = await doneColumn.evaluate(el => 
      window.getComputedStyle(el).backgroundColor
    );
    console.log('Initial background:', initialBg);
    
    // Get initial style attribute
    const initialStyle = await doneColumn.evaluate(el => el.style.backgroundColor);
    console.log('Initial style.backgroundColor:', initialStyle);
    
    // Hover over Done column
    console.log('Hovering over Done column...');
    await doneColumn.hover();
    await page.waitForTimeout(200);
    
    // Check background changed
    const hoverBg = await doneColumn.evaluate(el => 
      window.getComputedStyle(el).backgroundColor
    );
    console.log('Hover background (computed):', hoverBg);
    
    const hoverStyle = await doneColumn.evaluate(el => el.style.backgroundColor);
    console.log('Hover style.backgroundColor:', hoverStyle);
    
    // Check if drag-over class is added
    const hasDragOver = await doneColumn.evaluate(el => el.classList.contains('drag-over'));
    console.log('Has drag-over class:', hasDragOver);
    
    // Get the actual background from the element
    const actualBg = await doneColumn.evaluate(el => {
        const style = window.getComputedStyle(el);
        return {
            backgroundColor: style.backgroundColor,
            background: style.background,
            classList: Array.from(el.classList),
            inlineStyle: el.style.backgroundColor
        };
    });
    console.log('Actual element state:', actualBg);
    
    await page.waitForTimeout(2000);
    await page.mouse.up();
    await browser.close();
})();