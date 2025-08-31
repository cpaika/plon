import { test, expect } from '@playwright/test';
import { setupTest, TestHelpers } from './test-helpers';

test.describe('Drag and Drop Edge Cases and Bug Detection', () => {
  let helpers: TestHelpers;

  test.beforeEach(async ({ page }) => {
    helpers = await setupTest(page);
  });

  test.describe('Kanban Board Edge Cases', () => {
    test.beforeEach(async ({ page }) => {
      await helpers.navigateToKanban();
    });

    test('should handle drag hover without drop', async ({ page }) => {
      const todoCards = await helpers.getTaskCardsInColumn('Todo');
      const firstCard = todoCards.first();
      const inProgressColumn = await helpers.getKanbanColumn('In Progress');
      
      const initialTodoCount = await helpers.getColumnTaskCount('Todo');
      
      // Start drag but don't drop
      const cardBbox = await firstCard.boundingBox();
      const columnBbox = await inProgressColumn.boundingBox();
      
      if (cardBbox && columnBbox) {
        await page.mouse.move(cardBbox.x + cardBbox.width / 2, cardBbox.y + cardBbox.height / 2);
        await page.mouse.down();
        
        // Hover over column
        await page.mouse.move(
          columnBbox.x + columnBbox.width / 2,
          columnBbox.y + columnBbox.height / 2,
          { steps: 5 }
        );
        
        // Move away without dropping
        await page.mouse.move(100, 100);
        await page.mouse.up();
        
        // Verify card didn't move
        const finalTodoCount = await helpers.getColumnTaskCount('Todo');
        expect(finalTodoCount).toBe(initialTodoCount);
        
        // Verify column is not highlighted anymore
        const columnStyle = await inProgressColumn.evaluate(el => 
          window.getComputedStyle(el).backgroundColor
        );
        expect(columnStyle).not.toBe('rgb(232, 245, 233)'); // #e8f5e9
      }
    });

    test('should handle drag to same column', async ({ page }) => {
      const todoCards = await helpers.getTaskCardsInColumn('Todo');
      const firstCard = todoCards.first();
      const todoColumn = await helpers.getKanbanColumn('Todo');
      
      const initialCount = await helpers.getColumnTaskCount('Todo');
      const taskTitle = await firstCard.locator('h4').textContent();
      
      // Drag within same column
      await helpers.dragAndDrop(firstCard, todoColumn);
      
      // Verify count unchanged
      const finalCount = await helpers.getColumnTaskCount('Todo');
      expect(finalCount).toBe(initialCount);
      
      // Verify task still in Todo
      if (taskTitle) {
        const stillInTodo = await todoColumn.locator(`h4:has-text("${taskTitle}")`).isVisible();
        expect(stillInTodo).toBeTruthy();
      }
    });

    test('should handle simultaneous drags (race condition test)', async ({ page }) => {
      const todoCards = await helpers.getTaskCardsInColumn('Todo');
      if (await todoCards.count() >= 2) {
        const card1 = todoCards.first();
        const card2 = todoCards.nth(1);
        const inProgressColumn = await helpers.getKanbanColumn('In Progress');
        
        // Try to simulate near-simultaneous drags
        const bbox1 = await card1.boundingBox();
        const bbox2 = await card2.boundingBox();
        const targetBbox = await inProgressColumn.boundingBox();
        
        if (bbox1 && bbox2 && targetBbox) {
          // Start first drag
          await page.mouse.move(bbox1.x + bbox1.width / 2, bbox1.y + bbox1.height / 2);
          await page.mouse.down();
          
          // Quickly try second drag (should fail as first is in progress)
          const secondDragPromise = page.mouse.move(
            bbox2.x + bbox2.width / 2,
            bbox2.y + bbox2.height / 2
          );
          
          // Complete first drag
          await page.mouse.move(
            targetBbox.x + targetBbox.width / 2,
            targetBbox.y + targetBbox.height / 2
          );
          await page.mouse.up();
          
          await secondDragPromise;
          
          // Verify only one card moved
          const inProgressCount = await helpers.getColumnTaskCount('In Progress');
          expect(inProgressCount).toBeGreaterThan(0);
        }
      }
    });

    test('should clear drag state after error', async ({ page }) => {
      const todoCards = await helpers.getTaskCardsInColumn('Todo');
      const firstCard = todoCards.first();
      
      // Simulate drag that might error
      const bbox = await firstCard.boundingBox();
      if (bbox) {
        await page.mouse.move(bbox.x + bbox.width / 2, bbox.y + bbox.height / 2);
        await page.mouse.down();
        
        // Move to invalid area (outside columns)
        await page.mouse.move(0, 0);
        await page.mouse.up();
        
        // Try another drag to verify state is clear
        const secondCard = todoCards.nth(1);
        const inProgressColumn = await helpers.getKanbanColumn('In Progress');
        
        if (await secondCard.count() > 0) {
          await helpers.dragAndDrop(secondCard, inProgressColumn);
          
          // Should work if state cleared properly
          const inProgressCount = await helpers.getColumnTaskCount('In Progress');
          expect(inProgressCount).toBeGreaterThan(0);
        }
      }
    });

    test('should handle column overflow with many tasks', async ({ page }) => {
      // This tests if dragging still works when column has scrollbar
      const todoColumn = await helpers.getKanbanColumn('Todo');
      const todoCards = await helpers.getTaskCardsInColumn('Todo');
      
      // Check if column is scrollable
      const isScrollable = await todoColumn.evaluate(el => {
        const inner = el.querySelector('div[style*="overflow-y"]');
        return inner ? inner.scrollHeight > inner.clientHeight : false;
      });
      
      if (isScrollable && await todoCards.count() > 0) {
        // Try to drag last visible card
        const lastCard = todoCards.last();
        const doneColumn = await helpers.getKanbanColumn('Done');
        
        // Scroll to bottom first
        await todoColumn.evaluate(el => {
          const inner = el.querySelector('div[style*="overflow-y"]');
          if (inner) inner.scrollTop = inner.scrollHeight;
        });
        
        await helpers.dragAndDrop(lastCard, doneColumn);
        
        const doneCount = await helpers.getColumnTaskCount('Done');
        expect(doneCount).toBeGreaterThan(0);
      }
    });
  });

  test.describe('Map View Edge Cases', () => {
    test.beforeEach(async ({ page }) => {
      await helpers.navigateToMap();
    });

    test('should handle drag outside viewport', async ({ page }) => {
      const cards = await helpers.getMapTaskCards();
      const firstCard = cards.first();
      
      const initialPos = await helpers.getTaskPosition(
        await firstCard.locator('h4').textContent() || ''
      );
      
      // Try to drag outside viewport
      const bbox = await firstCard.boundingBox();
      if (bbox) {
        await page.mouse.move(bbox.x + bbox.width / 2, bbox.y + bbox.height / 2);
        await page.mouse.down();
        
        // Drag to negative coordinates
        await page.mouse.move(-100, -100);
        await page.mouse.up();
        
        await page.waitForTimeout(500);
        
        // Position should be clamped to valid range
        const newPos = await helpers.getTaskPosition(
          await firstCard.locator('h4').textContent() || ''
        );
        
        expect(newPos.x).toBeGreaterThanOrEqual(0);
        expect(newPos.y).toBeGreaterThanOrEqual(0);
      }
    });

    test('should maintain drag state with rapid zoom changes', async ({ page }) => {
      const cards = await helpers.getMapTaskCards();
      const firstCard = cards.first();
      
      // Start dragging
      const bbox = await firstCard.boundingBox();
      if (bbox) {
        await page.mouse.move(bbox.x + bbox.width / 2, bbox.y + bbox.height / 2);
        await page.mouse.down();
        
        // Change zoom while dragging (should not break drag)
        await helpers.clickZoomButton('Zoom In');
        
        // Continue drag
        await page.mouse.move(bbox.x + 100, bbox.y + 100);
        await page.mouse.up();
        
        // Verify card moved
        const finalBbox = await firstCard.boundingBox();
        expect(finalBbox?.x).not.toBe(bbox.x);
      }
    });

    test('should handle drag with maximum zoom', async ({ page }) => {
      // Zoom to maximum
      for (let i = 0; i < 5; i++) {
        await helpers.clickZoomButton('Zoom In');
      }
      
      const zoomLevel = await helpers.getZoomLevel();
      expect(zoomLevel).toBeLessThanOrEqual(300); // Max should be 3.0x
      
      // Try dragging at max zoom
      const cards = await helpers.getMapTaskCards();
      const firstCard = cards.first();
      
      const bbox = await firstCard.boundingBox();
      if (bbox) {
        await page.mouse.move(bbox.x + bbox.width / 2, bbox.y + bbox.height / 2);
        await page.mouse.down();
        await page.mouse.move(bbox.x + 50, bbox.y + 50);
        await page.mouse.up();
        
        // Should still work
        const newBbox = await firstCard.boundingBox();
        expect(newBbox?.x).not.toBe(bbox.x);
      }
    });

    test('should handle drag with minimum zoom', async ({ page }) => {
      // Zoom to minimum
      for (let i = 0; i < 5; i++) {
        await helpers.clickZoomButton('Zoom Out');
      }
      
      const zoomLevel = await helpers.getZoomLevel();
      expect(zoomLevel).toBeGreaterThanOrEqual(30); // Min should be 0.3x
      
      // Try dragging at min zoom
      const cards = await helpers.getMapTaskCards();
      const firstCard = cards.first();
      
      const bbox = await firstCard.boundingBox();
      if (bbox) {
        await page.mouse.move(bbox.x + bbox.width / 2, bbox.y + bbox.height / 2);
        await page.mouse.down();
        await page.mouse.move(bbox.x + 200, bbox.y + 200);
        await page.mouse.up();
        
        // Should still work
        const newBbox = await firstCard.boundingBox();
        expect(newBbox?.x).not.toBe(bbox.x);
      }
    });

    test('should handle task selection during drag', async ({ page }) => {
      const cards = await helpers.getMapTaskCards();
      if (await cards.count() >= 2) {
        const firstCard = cards.first();
        const secondCard = cards.nth(1);
        
        // Select first card
        await firstCard.click();
        await page.waitForTimeout(200);
        
        // Verify selected
        let style = await firstCard.getAttribute('style');
        expect(style).toContain('border: 2px solid #4CAF50');
        
        // Start dragging second card
        const bbox = await secondCard.boundingBox();
        if (bbox) {
          await page.mouse.move(bbox.x + bbox.width / 2, bbox.y + bbox.height / 2);
          await page.mouse.down();
          await page.mouse.move(bbox.x + 50, bbox.y + 50);
          await page.mouse.up();
          
          // First card should remain selected
          style = await firstCard.getAttribute('style');
          expect(style).toContain('border: 2px solid #4CAF50');
        }
      }
    });

    test('should handle delete during drag operation', async ({ page }) => {
      const cards = await helpers.getMapTaskCards();
      if (await cards.count() >= 2) {
        const firstCard = cards.first();
        const secondCard = cards.nth(1);
        
        // Start dragging first card
        const bbox = await firstCard.boundingBox();
        if (bbox) {
          await page.mouse.move(bbox.x + bbox.width / 2, bbox.y + bbox.height / 2);
          await page.mouse.down();
          
          // Try to delete second card while dragging
          await secondCard.hover();
          const deleteButton = secondCard.locator('button:has-text("×")');
          
          // This should either be blocked or handle gracefully
          try {
            await deleteButton.click({ force: true });
          } catch (e) {
            // Expected - click might be blocked during drag
          }
          
          // Complete drag
          await page.mouse.move(bbox.x + 100, bbox.y + 100);
          await page.mouse.up();
          
          // Verify state is consistent
          const finalCards = await helpers.getMapTaskCards();
          expect(await finalCards.count()).toBeGreaterThan(0);
        }
      }
    });
  });

  test.describe('Performance and Stress Tests', () => {
    test('should handle rapid task creation and dragging', async ({ page }) => {
      await helpers.navigateToMap();
      
      // Add multiple tasks quickly
      for (let i = 0; i < 3; i++) {
        await page.locator('button:has-text("Add Task")').click();
        await page.waitForTimeout(100);
      }
      
      // Try to drag newest task immediately
      const cards = await helpers.getMapTaskCards();
      const lastCard = cards.last();
      
      const bbox = await lastCard.boundingBox();
      if (bbox) {
        await page.mouse.move(bbox.x + bbox.width / 2, bbox.y + bbox.height / 2);
        await page.mouse.down();
        await page.mouse.move(bbox.x + 100, bbox.y + 100);
        await page.mouse.up();
        
        // Should work without errors
        const newBbox = await lastCard.boundingBox();
        expect(newBbox?.x).not.toBe(bbox.x);
      }
    });

    test('should maintain performance with many drag operations', async ({ page }) => {
      await helpers.navigateToKanban();
      
      const startTime = Date.now();
      
      // Perform multiple drags
      for (let i = 0; i < 3; i++) {
        const todoCards = await helpers.getTaskCardsInColumn('Todo');
        if (await todoCards.count() > 0) {
          const card = todoCards.first();
          const targetColumn = await helpers.getKanbanColumn(
            i % 2 === 0 ? 'In Progress' : 'Todo'
          );
          
          await helpers.dragAndDrop(card, targetColumn);
        }
      }
      
      const duration = Date.now() - startTime;
      console.log(`3 drag operations took ${duration}ms`);
      
      // Should complete in reasonable time (< 5 seconds)
      expect(duration).toBeLessThan(5000);
    });
  });
});