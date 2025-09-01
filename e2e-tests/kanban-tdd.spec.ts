import { test, expect, Page } from '@playwright/test';

test.describe('Kanban TDD - Bug Detection Tests', () => {
  let page: Page;

  test.beforeEach(async ({ page: testPage }) => {
    page = testPage;
    // Serve the test HTML file directly
    await page.goto(`file://${process.cwd()}/test-kanban.html`);
    await page.waitForLoadState('networkidle');
    await page.waitForSelector('h2:has-text("Kanban Board")');
  });

  test('Bug #1: Card should actually move to dropped column', async () => {
    // Get first task from Todo column
    const todoColumn = page.locator('.kanban-column[data-status="todo"]').first();
    const todoCard = todoColumn.locator('.kanban-card').first();
    
    // Get card title for tracking
    const cardTitle = await todoCard.locator('h4').textContent();
    console.log('Moving card:', cardTitle);
    
    // Get initial counts
    const initialTodoCount = await todoColumn.locator('.count').first().textContent();
    
    // Get In Progress column
    const inProgressColumn = page.locator('.kanban-column[data-status="in-progress"]').first();
    const initialInProgressCount = await inProgressColumn.locator('.count').first().textContent();
    
    // Perform drag and drop
    await todoCard.hover();
    await page.mouse.down();
    await inProgressColumn.hover();
    await page.mouse.up();
    
    // Wait for DOM update
    await page.waitForTimeout(500);
    
    // Verify card moved
    const newTodoCount = await todoColumn.locator('.count').first().textContent();
    const newInProgressCount = await inProgressColumn.locator('.count').first().textContent();
    
    // These assertions will likely fail initially
    expect(parseInt(newTodoCount!)).toBe(parseInt(initialTodoCount!) - 1);
    expect(parseInt(newInProgressCount!)).toBe(parseInt(initialInProgressCount!) + 1);
    
    // Verify card is in new column
    const inProgressCards = await inProgressColumn.locator('h4').allTextContents();
    expect(inProgressCards).toContain(cardTitle);
  });

  test('Bug #2: Column should highlight when dragging over it', async () => {
    const todoCard = page.locator('.kanban-card').first();
    const doneColumn = page.locator('.kanban-column[data-status="done"]').first();
    
    // Start dragging
    await todoCard.hover();
    await page.mouse.down();
    
    // Get initial background
    const initialBg = await doneColumn.evaluate(el => 
      window.getComputedStyle(el).backgroundColor
    );
    console.log('Initial background:', initialBg);
    
    // Hover over Done column
    await doneColumn.hover();
    await page.waitForTimeout(200);
    
    // Check that drag-over class is applied and background changed
    const hasDragOver = await doneColumn.evaluate(el => el.classList.contains('drag-over'));
    expect(hasDragOver).toBe(true);
    
    // Also check background (may be rgb or rgba)
    const hoverBg = await doneColumn.evaluate(el => 
      window.getComputedStyle(el).backgroundColor
    );
    console.log('Hover background:', hoverBg);
    
    // Should have changed from initial (accept rgb or rgba format)
    expect(hoverBg).not.toBe(initialBg);
    expect(hoverBg).toMatch(/rgb/);
    
    // Move away
    await page.mouse.move(100, 100);
    await page.waitForTimeout(100);
    
    // Background should revert
    const afterBg = await doneColumn.evaluate(el => 
      window.getComputedStyle(el).backgroundColor
    );
    expect(afterBg).toBe(initialBg);
    
    await page.mouse.up();
  });

  test('Bug #3: Dragging should work with native HTML5 drag events', async () => {
    const todoCard = page.locator('.kanban-card').first();
    const reviewColumn = page.locator('.kanban-column[data-status="review"]').first();
    
    const cardTitle = await todoCard.locator('h4').textContent();
    
    // Test that draggable attribute exists and is true
    const isDraggable = await todoCard.getAttribute('draggable');
    expect(isDraggable).toBe('true');
    
    // Use mouse events as a proxy for drag events (since DataTransfer can't be created in tests)
    await todoCard.hover();
    await page.mouse.down();
    await reviewColumn.hover();
    await page.mouse.up();
    
    await page.waitForTimeout(500);
    
    // Card should be in Review column
    const reviewCards = await reviewColumn.locator('h4').allTextContents();
    expect(reviewCards).toContain(cardTitle);
  });

  test('Bug #4: Multiple rapid drags should not duplicate or lose cards', async () => {
    // Count total cards initially
    const allCards = page.locator('.kanban-card');
    const initialTotalCards = await allCards.count();
    
    // Perform 3 rapid drags
    for (let i = 0; i < 3; i++) {
      const card = page.locator('.kanban-card').first();
      const targetColumn = page.locator(`div:has(h3:has-text("${i % 2 === 0 ? 'In Progress' : 'Todo'}"))`)
        .first();
      
      await card.hover();
      await page.mouse.down();
      await targetColumn.hover();
      await page.mouse.up();
      // Don't wait between drags
    }
    
    await page.waitForTimeout(500);
    
    // Total cards should remain the same
    const finalTotalCards = await allCards.count();
    expect(finalTotalCards).toBe(initialTotalCards);
  });

  test('Bug #5: Drag preview should show card being dragged', async () => {
    const todoCard = page.locator('.kanban-card').first();
    
    // Check initial opacity
    const initialOpacity = await todoCard.evaluate(el => 
      window.getComputedStyle(el).opacity
    );
    expect(initialOpacity).toBe('1');
    
    // Start dragging
    await todoCard.hover();
    await page.mouse.down();
    await page.mouse.move(200, 200);
    await page.waitForTimeout(100);
    
    // Check opacity during drag
    const dragOpacity = await todoCard.evaluate(el => 
      window.getComputedStyle(el).opacity
    );
    
    // Should be semi-transparent
    expect(parseFloat(dragOpacity)).toBeLessThan(1);
    expect(parseFloat(dragOpacity)).toBeGreaterThan(0);
    
    await page.mouse.up();
  });

  test('Bug #6: Empty column should accept dropped cards', async () => {
    // Find an empty column or make one empty
    const blockedColumn = page.locator('.kanban-column[data-status="blocked"]').first();
    const blockedCards = blockedColumn.locator('.kanban-card');
    
    // Clear blocked column if it has cards
    const blockedCount = await blockedCards.count();
    for (let i = 0; i < blockedCount; i++) {
      const deleteBtn = blockedColumn.locator('button:has-text("×")').first();
      await deleteBtn.click();
      await page.waitForTimeout(100);
    }
    
    // Now drag a card to empty column
    const todoCard = page.locator('.kanban-card').first();
    const cardTitle = await todoCard.locator('h4').textContent();
    
    await todoCard.hover();
    await page.mouse.down();
    await blockedColumn.hover();
    await page.mouse.up();
    
    await page.waitForTimeout(500);
    
    // Card should be in Blocked column
    const newBlockedCards = await blockedColumn.locator('h4').allTextContents();
    expect(newBlockedCards).toContain(cardTitle);
    
    // "Drop tasks here" should be gone
    const dropPlaceholder = blockedColumn.locator('text="Drop tasks here"');
    await expect(dropPlaceholder).not.toBeVisible();
  });

  test('Bug #7: Drag should be cancelable with ESC key', async () => {
    const todoCard = page.locator('.kanban-card').first();
    const todoColumn = page.locator('.kanban-column[data-status="todo"]').first();
    const initialCount = await todoColumn.locator('.count').first().textContent();
    
    // Start dragging
    await todoCard.hover();
    await page.mouse.down();
    await page.mouse.move(300, 300);
    
    // Press ESC
    await page.keyboard.press('Escape');
    
    await page.mouse.up();
    await page.waitForTimeout(500);
    
    // Card should still be in Todo
    const finalCount = await todoColumn.locator('.count').first().textContent();
    expect(finalCount).toBe(initialCount);
  });

  test('Bug #8: Dragging over non-column areas should not break state', async () => {
    const todoCard = page.locator('.kanban-card').first();
    const todoColumn = page.locator('.kanban-column[data-status="todo"]').first();
    const initialCount = await todoColumn.locator('.count').first().textContent();
    
    // Start dragging
    await todoCard.hover();
    await page.mouse.down();
    
    // Move to header area (non-droppable)
    await page.mouse.move(100, 50);
    await page.waitForTimeout(100);
    
    // Move to empty space between columns
    await page.mouse.move(500, 400);
    await page.waitForTimeout(100);
    
    // Release outside any column
    await page.mouse.up();
    
    await page.waitForTimeout(500);
    
    // Card should still be in original column
    const finalCount = await todoColumn.locator('.count').first().textContent();
    expect(finalCount).toBe(initialCount);
    
    // Should be able to drag again
    await todoCard.hover();
    await page.mouse.down();
    const inProgressColumn = page.locator('.kanban-column[data-status="in-progress"]').first();
    await inProgressColumn.hover();
    await page.mouse.up();
    
    await page.waitForTimeout(500);
    
    // This second drag should work
    const newCount = await todoColumn.locator('.count').first().textContent();
    expect(parseInt(newCount!)).toBe(parseInt(initialCount!) - 1);
  });

  test('Bug #9: Card details should remain intact after drag', async () => {
    const todoCard = page.locator('.kanban-card').first();
    
    // Get card details
    const title = await todoCard.locator('h4').textContent();
    const description = await todoCard.locator('p').textContent().catch(() => '');
    const hasPriority = await todoCard.locator('.priority-badge').count() > 0;
    
    // Drag to Done column
    const doneColumn = page.locator('.kanban-column[data-status="done"]').first();
    
    await todoCard.hover();
    await page.mouse.down();
    await doneColumn.hover();
    await page.mouse.up();
    
    await page.waitForTimeout(500);
    
    // Find the card in Done column
    const doneCard = doneColumn.locator('.kanban-card').filter({ hasText: title }).first();
    
    // Verify all details preserved
    expect(await doneCard.locator('h4').textContent()).toBe(title);
    if (description) {
      expect(await doneCard.locator('p').textContent()).toBe(description);
    }
    expect(await doneCard.locator('.priority-badge').count() > 0).toBe(hasPriority);
  });

  test('Bug #10: Drag should work after page interactions', async () => {
    // First interact with other elements
    const todoColumn = page.locator('.kanban-column[data-status="todo"]').first();
    
    // Delete a card first
    const deleteBtn = todoColumn.locator('button:has-text("×")').first();
    await deleteBtn.click();
    await page.waitForTimeout(200);
    
    // Now try to drag
    const todoCard = todoColumn.locator('.kanban-card').first();
    const cardTitle = await todoCard.locator('h4').textContent();
    
    const inProgressColumn = page.locator('.kanban-column[data-status="in-progress"]').first();
    
    await todoCard.hover();
    await page.mouse.down();
    await inProgressColumn.hover();
    await page.mouse.up();
    
    await page.waitForTimeout(500);
    
    // Should still work
    const inProgressCards = await inProgressColumn.locator('h4').allTextContents();
    expect(inProgressCards).toContain(cardTitle);
  });
});

test.describe('Kanban TDD - Performance Tests', () => {
  test('Should handle 10 rapid consecutive drags without breaking', async ({ page }) => {
    await page.goto(`file://${process.cwd()}/test-kanban.html`);
    await page.waitForLoadState('networkidle');
    
    const startTime = Date.now();
    
    for (let i = 0; i < 10; i++) {
      const card = page.locator('.kanban-card').first();
      const column = page.locator(`div:has(h3:has-text("${
        ['Todo', 'In Progress', 'Review', 'Done'][i % 4]
      }"))`)
        .first();
      
      await card.hover();
      await page.mouse.down();
      await column.hover();
      await page.mouse.up();
    }
    
    const duration = Date.now() - startTime;
    
    // Should complete in reasonable time
    expect(duration).toBeLessThan(10000);
    
    // All cards should still exist
    const totalCards = await page.locator('.kanban-card').count();
    expect(totalCards).toBeGreaterThan(0);
  });
});