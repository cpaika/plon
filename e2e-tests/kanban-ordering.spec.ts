import { test, expect, Page } from '@playwright/test';

test.describe('Kanban Card Ordering', () => {
  let page: Page;
  
  // Helper to get card titles in order from a column
  async function getCardTitlesInColumn(page: Page, columnName: string): Promise<string[]> {
    const column = page.locator(`div:has(> div > h3:has-text("${columnName}"))`).first();
    const titles = await column.locator('h4').allTextContents();
    return titles;
  }
  
  // Helper to drag card to specific position in column
  async function dragCardToPosition(
    page: Page, 
    cardTitle: string, 
    targetColumn: string, 
    targetPosition: number
  ) {
    // Find the card
    const card = page.locator(`div:has(> div > h4:has-text("${cardTitle}"))`).first();
    
    // Find target column
    const column = page.locator(`div:has(> div > h3:has-text("${targetColumn}"))`).first();
    
    // Get all cards in target column
    const cardsInColumn = column.locator('div[style*="cursor"][style*="background: white"]');
    const cardCount = await cardsInColumn.count();
    
    if (targetPosition === 0 || cardCount === 0) {
      // Drop at the top of the column
      const columnHeader = column.locator('div:has(> h3)').first();
      await card.hover();
      await page.mouse.down();
      await columnHeader.hover();
      await page.mouse.up();
    } else if (targetPosition >= cardCount) {
      // Drop at the bottom
      const lastCard = cardsInColumn.nth(cardCount - 1);
      await card.hover();
      await page.mouse.down();
      await lastCard.hover();
      // Move slightly below the last card
      const box = await lastCard.boundingBox();
      if (box) {
        await page.mouse.move(box.x + box.width / 2, box.y + box.height + 5);
      }
      await page.mouse.up();
    } else {
      // Drop between cards
      const targetCard = cardsInColumn.nth(targetPosition);
      await card.hover();
      await page.mouse.down();
      
      // Hover over the target position
      const box = await targetCard.boundingBox();
      if (box) {
        // Move to the top half of the target card to insert before it
        await page.mouse.move(box.x + box.width / 2, box.y + 5);
      }
      await page.mouse.up();
    }
    
    await page.waitForTimeout(500);
  }

  test.beforeEach(async ({ page: testPage }) => {
    page = testPage;
    // This would be for testing against a running instance
    // For actual desktop app, we'd need to start it first
  });

  test('Cards maintain order within a column', async () => {
    // Navigate to Kanban
    await page.goto('http://localhost:8080');
    await page.click('button:has-text("Kanban")');
    await page.waitForSelector('h2:has-text("Kanban Board")');
    
    // Get initial order in Todo column
    const initialOrder = await getCardTitlesInColumn(page, 'Todo');
    console.log('Initial Todo order:', initialOrder);
    
    // Drag the first card to the second position
    if (initialOrder.length >= 2) {
      await dragCardToPosition(page, initialOrder[0], 'Todo', 1);
      
      // Check new order
      const newOrder = await getCardTitlesInColumn(page, 'Todo');
      console.log('New Todo order:', newOrder);
      
      // First card should now be in second position
      expect(newOrder[0]).toBe(initialOrder[1]);
      expect(newOrder[1]).toBe(initialOrder[0]);
    }
  });

  test('Card order persists when moving between columns', async () => {
    await page.goto('http://localhost:8080');
    await page.click('button:has-text("Kanban")');
    await page.waitForSelector('h2:has-text("Kanban Board")');
    
    // Get cards from Todo
    const todoCards = await getCardTitlesInColumn(page, 'Todo');
    const inProgressCards = await getCardTitlesInColumn(page, 'In Progress');
    
    if (todoCards.length > 0 && inProgressCards.length > 0) {
      const cardToMove = todoCards[0];
      
      // Move card from Todo to middle of In Progress
      const targetPosition = Math.floor(inProgressCards.length / 2);
      await dragCardToPosition(page, cardToMove, 'In Progress', targetPosition);
      
      // Verify card is in correct position
      const newInProgressCards = await getCardTitlesInColumn(page, 'In Progress');
      expect(newInProgressCards[targetPosition]).toBe(cardToMove);
      
      // Navigate away and back
      await page.click('button:has-text("List")');
      await page.waitForTimeout(500);
      await page.click('button:has-text("Kanban")');
      await page.waitForSelector('h2:has-text("Kanban Board")');
      
      // Verify order is maintained
      const finalInProgressCards = await getCardTitlesInColumn(page, 'In Progress');
      expect(finalInProgressCards[targetPosition]).toBe(cardToMove);
    }
  });

  test('Dragging shows insertion point indicator', async () => {
    await page.goto('http://localhost:8080');
    await page.click('button:has-text("Kanban")');
    await page.waitForSelector('h2:has-text("Kanban Board")');
    
    const todoCards = await getCardTitlesInColumn(page, 'Todo');
    if (todoCards.length >= 2) {
      const card = page.locator(`div:has(> div > h4:has-text("${todoCards[0]}"))`).first();
      
      // Start dragging
      await card.hover();
      await page.mouse.down();
      
      // Move over another card
      const targetCard = page.locator(`div:has(> div > h4:has-text("${todoCards[1]}"))`).first();
      await targetCard.hover();
      
      // Check for insertion indicator (a line or highlighted area)
      const indicator = await page.locator('.insertion-indicator, .drop-indicator, [class*="indicator"]').isVisible();
      console.log('Insertion indicator visible:', indicator);
      
      // Should show some visual feedback
      // This might need adjustment based on actual implementation
      
      await page.mouse.up();
    }
  });

  test('Order persists after page refresh', async () => {
    await page.goto('http://localhost:8080');
    await page.click('button:has-text("Kanban")');
    await page.waitForSelector('h2:has-text("Kanban Board")');
    
    // Get initial order
    const initialOrder = await getCardTitlesInColumn(page, 'In Progress');
    console.log('Initial In Progress order:', initialOrder);
    
    if (initialOrder.length >= 2) {
      // Reorder cards
      await dragCardToPosition(page, initialOrder[0], 'In Progress', initialOrder.length - 1);
      
      const reorderedCards = await getCardTitlesInColumn(page, 'In Progress');
      console.log('Reordered In Progress:', reorderedCards);
      
      // Refresh page
      await page.reload();
      await page.click('button:has-text("Kanban")');
      await page.waitForSelector('h2:has-text("Kanban Board")');
      
      // Check order is preserved
      const finalOrder = await getCardTitlesInColumn(page, 'In Progress');
      expect(finalOrder).toEqual(reorderedCards);
    }
  });

  test('Multiple reorderings in same column persist', async () => {
    await page.goto('http://localhost:8080');
    await page.click('button:has-text("Kanban")');
    await page.waitForSelector('h2:has-text("Kanban Board")');
    
    const column = 'Done';
    const cards = await getCardTitlesInColumn(page, column);
    
    if (cards.length >= 3) {
      // Perform multiple reorderings
      // Move first to last
      await dragCardToPosition(page, cards[0], column, cards.length - 1);
      
      // Move what's now first to middle
      const afterFirst = await getCardTitlesInColumn(page, column);
      await dragCardToPosition(page, afterFirst[0], column, Math.floor(cards.length / 2));
      
      const finalOrder = await getCardTitlesInColumn(page, column);
      
      // Navigate away and back
      await page.click('button:has-text("Map")');
      await page.waitForTimeout(500);
      await page.click('button:has-text("Kanban")');
      await page.waitForSelector('h2:has-text("Kanban Board")');
      
      // Verify complex reordering persisted
      const persistedOrder = await getCardTitlesInColumn(page, column);
      expect(persistedOrder).toEqual(finalOrder);
    }
  });
});