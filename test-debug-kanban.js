const { chromium } = require('playwright');

(async () => {
    const browser = await chromium.launch({ headless: false });
    const page = await browser.newPage();
    
    // Enable console logging
    page.on('console', msg => console.log('PAGE LOG:', msg.text()));
    
    // Load the kanban HTML
    await page.goto(`file://${process.cwd()}/test-kanban.html`);
    
    // Get initial counts
    const todoColumn = page.locator('.kanban-column.todo').first();
    const inProgressColumn = page.locator('.kanban-column.in-progress').first();
    
    const initialTodoCount = await todoColumn.locator('.count').first().textContent();
    const initialInProgressCount = await inProgressColumn.locator('.count').first().textContent();
    
    console.log('Initial Todo count:', initialTodoCount);
    console.log('Initial In Progress count:', initialInProgressCount);
    
    // Get the first card
    const todoCard = todoColumn.locator('.kanban-card').first();
    const cardTitle = await todoCard.locator('h4').textContent();
    console.log('Moving card:', cardTitle);
    
    // Perform drag and drop
    await todoCard.hover();
    await page.mouse.down();
    await inProgressColumn.hover();
    await page.mouse.up();
    
    // Wait for DOM update
    await page.waitForTimeout(1000);
    
    // Check new counts
    const newTodoCount = await todoColumn.locator('.count').first().textContent();
    const newInProgressCount = await inProgressColumn.locator('.count').first().textContent();
    
    console.log('New Todo count:', newTodoCount);
    console.log('New In Progress count:', newInProgressCount);
    
    // Check if card is in new column
    const inProgressCards = await inProgressColumn.locator('h4').allTextContents();
    console.log('Cards in In Progress:', inProgressCards);
    
    await page.waitForTimeout(3000);
    await browser.close();
})();