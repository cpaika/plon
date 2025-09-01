const { chromium } = require('playwright');

(async () => {
    const browser = await chromium.launch({ headless: false, slowMo: 100 });
    const page = await browser.newPage();
    
    // Load the kanban HTML
    await page.goto(`file://${process.cwd()}/test-kanban.html`);
    
    console.log('Testing drag improvements...\n');
    
    const todoCard = page.locator('.kanban-card').first();
    const inProgressColumn = page.locator('.kanban-column[data-status="in-progress"]').first();
    
    // Test 1: Check that text is not selectable
    console.log('1. Testing text selection prevention...');
    const isUserSelectNone = await todoCard.evaluate(el => {
        const style = window.getComputedStyle(el);
        return style.userSelect === 'none';
    });
    console.log('   Card has user-select: none?', isUserSelectNone);
    
    // Test 2: Start dragging and check visibility
    console.log('\n2. Testing card visibility during drag...');
    await todoCard.hover();
    await page.mouse.down();
    await page.waitForTimeout(100);
    
    const cardVisibility = await todoCard.evaluate(el => el.style.visibility);
    console.log('   Card visibility after drag start:', cardVisibility || 'visible');
    console.log('   Expected: hidden');
    
    // Test 3: Check if ghost exists
    console.log('\n3. Testing drag ghost...');
    const ghostExists = await page.locator('.drag-ghost').count() > 0;
    console.log('   Ghost element exists?', ghostExists);
    
    if (ghostExists) {
        const ghostStyle = await page.locator('.drag-ghost').evaluate(el => ({
            position: window.getComputedStyle(el).position,
            opacity: window.getComputedStyle(el).opacity,
            transform: window.getComputedStyle(el).transform
        }));
        console.log('   Ghost style:', ghostStyle);
    }
    
    // Test 4: Check body dragging class
    console.log('\n4. Testing body dragging class...');
    const bodyHasDraggingClass = await page.evaluate(() => document.body.classList.contains('dragging'));
    console.log('   Body has dragging class?', bodyHasDraggingClass);
    
    // Move to trigger drag
    await page.mouse.move(400, 300);
    await page.waitForTimeout(100);
    
    // Test 5: Complete the drag
    console.log('\n5. Completing drag...');
    await inProgressColumn.hover();
    await page.mouse.up();
    await page.waitForTimeout(100);
    
    // Check that card is visible again
    const cardVisibilityAfter = await todoCard.evaluate(el => el.style.visibility);
    console.log('   Card visibility after drop:', cardVisibilityAfter || 'visible');
    
    // Check that ghost is removed
    const ghostExistsAfter = await page.locator('.drag-ghost').count() > 0;
    console.log('   Ghost removed after drop?', !ghostExistsAfter);
    
    // Check that body dragging class is removed
    const bodyHasDraggingClassAfter = await page.evaluate(() => document.body.classList.contains('dragging'));
    console.log('   Body dragging class removed?', !bodyHasDraggingClassAfter);
    
    console.log('\n✅ All drag improvements tested!');
    
    await page.waitForTimeout(2000);
    await browser.close();
})();