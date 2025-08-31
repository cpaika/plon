import { test, expect, Page } from '@playwright/test';

test.describe('Kanban Persistence Tests', () => {
  let page: Page;

  test.beforeEach(async ({ page: testPage }) => {
    page = testPage;
    // Test against the actual Dioxus app
    await page.goto('http://localhost:8080');
    await page.waitForLoadState('networkidle');
    
    // Navigate to Kanban view
    await page.click('text=Kanban');
    await page.waitForSelector('h2:has-text("Kanban Board")');
  });

  test('Text selection should be prevented during drag', async () => {
    // Get a card to drag
    const todoColumn = page.locator('div:has(h3:has-text("Todo"))').first();
    const todoCard = todoColumn.locator('div[style*="cursor"]').first();
    
    // Get card text content
    const cardTitle = await todoCard.locator('h4').textContent();
    
    // Start dragging
    await todoCard.hover();
    await page.mouse.down();
    
    // Move mouse across text multiple times (this would normally select text)
    for (let i = 0; i < 5; i++) {
      await page.mouse.move(100, 100);
      await page.mouse.move(300, 100);
    }
    
    // Check that no text is selected
    const selection = await page.evaluate(() => window.getSelection()?.toString());
    expect(selection).toBe('');
    
    // Check user-select CSS property on container
    const userSelect = await page.locator('div[style*="padding: 20px"]').first().evaluate(el => {
      return window.getComputedStyle(el).userSelect;
    });
    expect(userSelect).toBe('none');
    
    // Release mouse
    await page.mouse.up();
  });

  test('Card moves should persist after page refresh', async () => {
    // Get first task from Todo column
    const todoColumn = page.locator('div:has(h3:has-text("Todo"))').first();
    const todoCard = todoColumn.locator('div[style*="cursor"]').first();
    
    // Get card title for tracking
    const cardTitle = await todoCard.locator('h4').textContent();
    console.log('Moving card:', cardTitle);
    
    // Get initial Todo count
    const initialTodoCount = await todoColumn.locator('span[style*="border-radius: 12px"]').first().textContent();
    
    // Get In Progress column
    const inProgressColumn = page.locator('div:has(h3:has-text("In Progress"))').first();
    const initialInProgressCount = await inProgressColumn.locator('span[style*="border-radius: 12px"]').first().textContent();
    
    // Perform drag and drop
    await todoCard.hover();
    await page.mouse.down();
    await inProgressColumn.hover();
    await page.mouse.up();
    
    // Wait for state update
    await page.waitForTimeout(500);
    
    // Verify card moved
    const newTodoCount = await todoColumn.locator('span[style*="border-radius: 12px"]').first().textContent();
    const newInProgressCount = await inProgressColumn.locator('span[style*="border-radius: 12px"]').first().textContent();
    
    expect(parseInt(newTodoCount!)).toBe(parseInt(initialTodoCount!) - 1);
    expect(parseInt(newInProgressCount!)).toBe(parseInt(initialInProgressCount!) + 1);
    
    // Refresh the page
    await page.reload();
    await page.waitForSelector('h2:has-text("Kanban Board")');
    
    // Re-get columns after refresh
    const todoColumnAfter = page.locator('div:has(h3:has-text("Todo"))').first();
    const inProgressColumnAfter = page.locator('div:has(h3:has-text("In Progress"))').first();
    
    // Verify counts are still the same after refresh
    const todoCountAfterRefresh = await todoColumnAfter.locator('span[style*="border-radius: 12px"]').first().textContent();
    const inProgressCountAfterRefresh = await inProgressColumnAfter.locator('span[style*="border-radius: 12px"]').first().textContent();
    
    expect(todoCountAfterRefresh).toBe(newTodoCount);
    expect(inProgressCountAfterRefresh).toBe(newInProgressCount);
    
    // Verify the specific card is still in In Progress column
    const inProgressCards = await inProgressColumnAfter.locator('h4').allTextContents();
    expect(inProgressCards).toContain(cardTitle);
  });

  test('Multiple card moves should all persist', async () => {
    // Move card from Todo to In Progress
    const todoColumn = page.locator('div:has(h3:has-text("Todo"))').first();
    const todoCard1 = todoColumn.locator('div[style*="cursor"]').first();
    const cardTitle1 = await todoCard1.locator('h4').textContent();
    
    const inProgressColumn = page.locator('div:has(h3:has-text("In Progress"))').first();
    
    await todoCard1.hover();
    await page.mouse.down();
    await inProgressColumn.hover();
    await page.mouse.up();
    await page.waitForTimeout(300);
    
    // Move another card from Todo to Done
    const todoCard2 = todoColumn.locator('div[style*="cursor"]').first();
    const cardTitle2 = await todoCard2.locator('h4').textContent();
    
    const doneColumn = page.locator('div:has(h3:has-text("Done"))').first();
    
    await todoCard2.hover();
    await page.mouse.down();
    await doneColumn.hover();
    await page.mouse.up();
    await page.waitForTimeout(300);
    
    // Move a card from In Progress to Review
    const inProgressCard = inProgressColumn.locator('div[style*="cursor"]').first();
    const cardTitle3 = await inProgressCard.locator('h4').textContent();
    
    const reviewColumn = page.locator('div:has(h3:has-text("Review"))').first();
    
    await inProgressCard.hover();
    await page.mouse.down();
    await reviewColumn.hover();
    await page.mouse.up();
    await page.waitForTimeout(300);
    
    // Refresh the page
    await page.reload();
    await page.waitForSelector('h2:has-text("Kanban Board")');
    
    // Verify all moves persisted
    const inProgressCardsAfter = await page.locator('div:has(h3:has-text("In Progress"))').first().locator('h4').allTextContents();
    const doneCardsAfter = await page.locator('div:has(h3:has-text("Done"))').first().locator('h4').allTextContents();
    const reviewCardsAfter = await page.locator('div:has(h3:has-text("Review"))').first().locator('h4').allTextContents();
    
    expect(inProgressCardsAfter).toContain(cardTitle1);
    expect(doneCardsAfter).toContain(cardTitle2);
    expect(reviewCardsAfter).toContain(cardTitle3);
  });
});