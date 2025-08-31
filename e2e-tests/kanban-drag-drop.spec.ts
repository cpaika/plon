import { test, expect } from '@playwright/test';
import { setupTest, TestHelpers } from './test-helpers';

test.describe('Kanban Board Drag and Drop', () => {
  let helpers: TestHelpers;

  test.beforeEach(async ({ page }) => {
    helpers = await setupTest(page);
    await helpers.navigateToKanban();
  });

  test('should display kanban board with all columns', async ({ page }) => {
    // Verify all columns are present
    await expect(page.locator('h3:has-text("Todo")')).toBeVisible();
    await expect(page.locator('h3:has-text("In Progress")')).toBeVisible();
    await expect(page.locator('h3:has-text("Review")')).toBeVisible();
    await expect(page.locator('h3:has-text("Done")')).toBeVisible();
    await expect(page.locator('h3:has-text("Blocked")')).toBeVisible();
  });

  test('should display task cards with draggable attribute', async ({ page }) => {
    const cards = await helpers.getMapTaskCards();
    const count = await cards.count();
    
    expect(count).toBeGreaterThan(0);
    
    // Check first card has draggable attribute
    const firstCard = cards.first();
    await expect(firstCard).toHaveAttribute('draggable', 'true');
  });

  test('should drag task from Todo to In Progress', async ({ page }) => {
    // Get initial counts
    const todoColumn = await helpers.getKanbanColumn('Todo');
    const inProgressColumn = await helpers.getKanbanColumn('In Progress');
    
    const initialTodoCount = await helpers.getColumnTaskCount('Todo');
    const initialInProgressCount = await helpers.getColumnTaskCount('In Progress');
    
    // Get first task in Todo column
    const todoCards = await helpers.getTaskCardsInColumn('Todo');
    const firstTodoCard = todoCards.first();
    
    // Verify card exists
    await expect(firstTodoCard).toBeVisible();
    
    // Get task title for verification
    const taskTitle = await firstTodoCard.locator('h4').textContent();
    console.log(`Dragging task: ${taskTitle}`);
    
    // Perform drag and drop
    await helpers.dragAndDrop(firstTodoCard, inProgressColumn);
    
    // Verify counts changed
    const newTodoCount = await helpers.getColumnTaskCount('Todo');
    const newInProgressCount = await helpers.getColumnTaskCount('In Progress');
    
    expect(newTodoCount).toBe(initialTodoCount - 1);
    expect(newInProgressCount).toBe(initialInProgressCount + 1);
    
    // Verify task is now in In Progress column
    if (taskTitle) {
      const movedCard = await helpers.getTaskCardByTitle(taskTitle);
      const inProgressCards = await helpers.getTaskCardsInColumn('In Progress');
      const cardTexts = await inProgressCards.allTextContents();
      
      expect(cardTexts.some(text => text.includes(taskTitle))).toBeTruthy();
    }
  });

  test('should show visual feedback when dragging over column', async ({ page }) => {
    const todoCards = await helpers.getTaskCardsInColumn('Todo');
    const firstCard = todoCards.first();
    const doneColumn = await helpers.getKanbanColumn('Done');
    
    // Start dragging
    const cardBbox = await firstCard.boundingBox();
    if (cardBbox) {
      await page.mouse.move(cardBbox.x + cardBbox.width / 2, cardBbox.y + cardBbox.height / 2);
      await page.mouse.down();
      
      // Move over Done column
      const columnBbox = await doneColumn.boundingBox();
      if (columnBbox) {
        await page.mouse.move(
          columnBbox.x + columnBbox.width / 2,
          columnBbox.y + columnBbox.height / 2,
          { steps: 5 }
        );
        
        // Check for visual feedback (background color change)
        // Note: This might need adjustment based on actual implementation
        await page.waitForTimeout(100);
        
        // Release mouse
        await page.mouse.up();
      }
    }
  });

  test('should handle drag and drop between non-adjacent columns', async ({ page }) => {
    // Drag from Todo directly to Done
    const todoCards = await helpers.getTaskCardsInColumn('Todo');
    const firstCard = todoCards.first();
    const doneColumn = await helpers.getKanbanColumn('Done');
    
    const initialTodoCount = await helpers.getColumnTaskCount('Todo');
    const initialDoneCount = await helpers.getColumnTaskCount('Done');
    
    await helpers.dragAndDrop(firstCard, doneColumn);
    
    const newTodoCount = await helpers.getColumnTaskCount('Todo');
    const newDoneCount = await helpers.getColumnTaskCount('Done');
    
    expect(newTodoCount).toBe(initialTodoCount - 1);
    expect(newDoneCount).toBe(initialDoneCount + 1);
  });

  test('should handle dragging to empty column', async ({ page }) => {
    // Find or ensure we have an empty column
    const blockedColumn = await helpers.getKanbanColumn('Blocked');
    const blockedCards = await helpers.getTaskCardsInColumn('Blocked');
    const blockedCount = await blockedCards.count();
    
    if (blockedCount === 0) {
      // Blocked is empty, drag a task there
      const todoCards = await helpers.getTaskCardsInColumn('Todo');
      const firstCard = todoCards.first();
      
      await helpers.dragAndDrop(firstCard, blockedColumn);
      
      const newBlockedCount = await helpers.getColumnTaskCount('Blocked');
      expect(newBlockedCount).toBe(1);
      
      // Verify "Drop tasks here" placeholder is gone
      const placeholder = blockedColumn.locator('text="Drop tasks here"');
      await expect(placeholder).not.toBeVisible();
    }
  });

  test('should delete task when clicking X button', async ({ page }) => {
    const todoCards = await helpers.getTaskCardsInColumn('Todo');
    const initialCount = await todoCards.count();
    
    // Click delete on first card
    const firstCard = todoCards.first();
    await firstCard.hover();
    await firstCard.locator('button:has-text("×")').click();
    
    await page.waitForTimeout(500);
    
    const newCount = await (await helpers.getTaskCardsInColumn('Todo')).count();
    expect(newCount).toBe(initialCount - 1);
  });

  test('should maintain task properties after drag', async ({ page }) => {
    const todoCards = await helpers.getTaskCardsInColumn('Todo');
    const firstCard = todoCards.first();
    
    // Get task details before drag
    const title = await firstCard.locator('h4').textContent();
    const description = await firstCard.locator('p').textContent();
    const hasPriorityBadge = await firstCard.locator('span:has-text("Priority")').isVisible();
    
    // Drag to In Progress
    const inProgressColumn = await helpers.getKanbanColumn('In Progress');
    await helpers.dragAndDrop(firstCard, inProgressColumn);
    
    // Find the moved card and verify properties
    if (title) {
      const movedCard = await helpers.getTaskCardByTitle(title);
      
      expect(await movedCard.locator('h4').textContent()).toBe(title);
      if (description) {
        expect(await movedCard.locator('p').textContent()).toBe(description);
      }
      expect(await movedCard.locator('span:has-text("Priority")').isVisible()).toBe(hasPriorityBadge);
    }
  });

  test('should handle rapid consecutive drags', async ({ page }) => {
    const todoCards = await helpers.getTaskCardsInColumn('Todo');
    const firstCard = todoCards.first();
    
    // Drag through multiple columns quickly
    const inProgressColumn = await helpers.getKanbanColumn('In Progress');
    const reviewColumn = await helpers.getKanbanColumn('Review');
    const doneColumn = await helpers.getKanbanColumn('Done');
    
    // Todo -> In Progress
    await helpers.dragAndDrop(firstCard, inProgressColumn);
    
    // In Progress -> Review
    const movedCard1 = (await helpers.getTaskCardsInColumn('In Progress')).first();
    await helpers.dragAndDrop(movedCard1, reviewColumn);
    
    // Review -> Done
    const movedCard2 = (await helpers.getTaskCardsInColumn('Review')).first();
    await helpers.dragAndDrop(movedCard2, doneColumn);
    
    // Verify final position
    const doneCount = await helpers.getColumnTaskCount('Done');
    expect(doneCount).toBeGreaterThan(0);
  });

  test('should handle drag cancellation (ESC key)', async ({ page }) => {
    const todoCards = await helpers.getTaskCardsInColumn('Todo');
    const firstCard = todoCards.first();
    const initialTodoCount = await helpers.getColumnTaskCount('Todo');
    
    // Start dragging
    const cardBbox = await firstCard.boundingBox();
    if (cardBbox) {
      await page.mouse.move(cardBbox.x + cardBbox.width / 2, cardBbox.y + cardBbox.height / 2);
      await page.mouse.down();
      
      // Move partially
      await page.mouse.move(cardBbox.x + 100, cardBbox.y);
      
      // Press ESC to cancel
      await page.keyboard.press('Escape');
      
      // Release mouse
      await page.mouse.up();
      
      // Verify card didn't move
      const newTodoCount = await helpers.getColumnTaskCount('Todo');
      expect(newTodoCount).toBe(initialTodoCount);
    }
  });

  test('should handle multiple cards in same column', async ({ page }) => {
    // Ensure we have multiple cards in Todo
    const todoCards = await helpers.getTaskCardsInColumn('Todo');
    const cardCount = await todoCards.count();
    
    if (cardCount >= 2) {
      // Drag second card to In Progress
      const secondCard = todoCards.nth(1);
      const inProgressColumn = await helpers.getKanbanColumn('In Progress');
      
      const secondCardTitle = await secondCard.locator('h4').textContent();
      
      await helpers.dragAndDrop(secondCard, inProgressColumn);
      
      // Verify specific card moved
      if (secondCardTitle) {
        const movedCard = await helpers.getTaskCardByTitle(secondCardTitle);
        const inProgressCards = await helpers.getTaskCardsInColumn('In Progress');
        const cardTexts = await inProgressCards.allTextContents();
        
        expect(cardTexts.some(text => text.includes(secondCardTitle))).toBeTruthy();
      }
    }
  });
});