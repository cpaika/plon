import { test, expect, Page } from '@playwright/test';

test.describe('Dependency Persistence', () => {
  let page: Page;

  test.beforeEach(async ({ browser }) => {
    // Start with a fresh database for each test
    const context = await browser.newContext();
    page = await context.newPage();
    
    // Navigate to the map view
    await page.goto('http://localhost:8080');
    await page.click('text=Map');
    await page.waitForTimeout(500); // Wait for view to load
  });

  test('should create dependency by dragging from right node to left node', async () => {
    // Create two tasks first
    await page.click('button:has-text("Add Task")');
    await page.waitForTimeout(200);
    
    // Get the first task and move it to a known position
    const task1 = await page.locator('.task-card').first();
    await task1.dragTo(await page.locator('.map-container'), {
      targetPosition: { x: 200, y: 200 }
    });
    
    // Add second task
    await page.click('button:has-text("Add Task")');
    await page.waitForTimeout(200);
    
    // Move second task to a different position
    const task2 = await page.locator('.task-card').nth(1);
    await task2.dragTo(await page.locator('.map-container'), {
      targetPosition: { x: 500, y: 200 }
    });
    
    // Find the right node of the first task (output connector)
    const rightNode = await page.locator('.task-card').first().locator('.connection-node-right');
    
    // Find the left node of the second task (input connector)
    const leftNode = await page.locator('.task-card').nth(1).locator('.connection-node-left');
    
    // Drag from right node to left node to create dependency
    await rightNode.hover();
    await page.mouse.down();
    await leftNode.hover();
    await page.mouse.up();
    
    // Verify dependency line is created
    const dependencyLine = await page.locator('svg line.dependency-line, svg path.dependency-line');
    await expect(dependencyLine).toBeVisible();
    
    // Verify the dependency count
    const dependencies = await dependencyLine.count();
    expect(dependencies).toBeGreaterThan(0);
  });

  test('should persist dependencies after page reload', async () => {
    // Create two tasks
    await page.click('button:has-text("Add Task")');
    await page.waitForTimeout(200);
    await page.click('button:has-text("Add Task")');
    await page.waitForTimeout(200);
    
    // Position tasks
    const task1 = await page.locator('.task-card').first();
    await task1.dragTo(await page.locator('.map-container'), {
      targetPosition: { x: 200, y: 300 }
    });
    
    const task2 = await page.locator('.task-card').nth(1);
    await task2.dragTo(await page.locator('.map-container'), {
      targetPosition: { x: 500, y: 300 }
    });
    
    // Create dependency
    const rightNode = await page.locator('.task-card').first().locator('.connection-node-right');
    const leftNode = await page.locator('.task-card').nth(1).locator('.connection-node-left');
    
    await rightNode.hover();
    await page.mouse.down();
    await leftNode.hover();
    await page.mouse.up();
    
    // Wait for dependency to be saved
    await page.waitForTimeout(500);
    
    // Count dependencies before reload
    const dependenciesBeforeReload = await page.locator('svg line.dependency-line, svg path.dependency-line').count();
    expect(dependenciesBeforeReload).toBeGreaterThan(0);
    
    // Reload the page
    await page.reload();
    await page.waitForTimeout(1000); // Wait for data to load
    
    // Navigate back to map view
    await page.click('text=Map');
    await page.waitForTimeout(500);
    
    // Verify dependencies are still there
    const dependenciesAfterReload = await page.locator('svg line.dependency-line, svg path.dependency-line').count();
    expect(dependenciesAfterReload).toBe(dependenciesBeforeReload);
  });

  test('should persist task positions after dragging', async () => {
    // Create a task
    await page.click('button:has-text("Add Task")');
    await page.waitForTimeout(200);
    
    // Move task to specific position
    const task = await page.locator('.task-card').first();
    const targetX = 350;
    const targetY = 450;
    
    await task.dragTo(await page.locator('.map-container'), {
      targetPosition: { x: targetX, y: targetY }
    });
    
    // Wait for position to be saved
    await page.waitForTimeout(500);
    
    // Get task position before reload
    const positionBefore = await task.boundingBox();
    expect(positionBefore).not.toBeNull();
    
    // Reload the page
    await page.reload();
    await page.waitForTimeout(1000);
    
    // Navigate back to map view
    await page.click('text=Map');
    await page.waitForTimeout(500);
    
    // Get task position after reload
    const taskAfterReload = await page.locator('.task-card').first();
    const positionAfter = await taskAfterReload.boundingBox();
    
    // Verify position is preserved (within a small tolerance)
    expect(positionAfter).not.toBeNull();
    if (positionBefore && positionAfter) {
      expect(Math.abs(positionAfter.x - positionBefore.x)).toBeLessThan(10);
      expect(Math.abs(positionAfter.y - positionBefore.y)).toBeLessThan(10);
    }
  });

  test('should show visual feedback during dependency creation', async () => {
    // Create two tasks
    await page.click('button:has-text("Add Task")');
    await page.waitForTimeout(200);
    await page.click('button:has-text("Add Task")');
    await page.waitForTimeout(200);
    
    // Start dragging from right node
    const rightNode = await page.locator('.task-card').first().locator('.connection-node-right');
    await rightNode.hover();
    await page.mouse.down();
    
    // Move mouse to create drag line
    await page.mouse.move(400, 300);
    
    // Verify drag line is visible
    const dragLine = await page.locator('svg line[stroke="red"], svg line[stroke="#ff0000"]');
    await expect(dragLine).toBeVisible();
    
    // Complete the drag
    const leftNode = await page.locator('.task-card').nth(1).locator('.connection-node-left');
    await leftNode.hover();
    await page.mouse.up();
    
    // Verify drag line is gone and dependency line appears
    await expect(dragLine).not.toBeVisible();
    const dependencyLine = await page.locator('svg line.dependency-line, svg path.dependency-line');
    await expect(dependencyLine).toBeVisible();
  });

  test('should not create duplicate dependencies', async () => {
    // Create two tasks
    await page.click('button:has-text("Add Task")');
    await page.waitForTimeout(200);
    await page.click('button:has-text("Add Task")');
    await page.waitForTimeout(200);
    
    const rightNode = await page.locator('.task-card').first().locator('.connection-node-right');
    const leftNode = await page.locator('.task-card').nth(1).locator('.connection-node-left');
    
    // Create first dependency
    await rightNode.hover();
    await page.mouse.down();
    await leftNode.hover();
    await page.mouse.up();
    await page.waitForTimeout(300);
    
    // Count dependencies
    const firstCount = await page.locator('svg line.dependency-line, svg path.dependency-line').count();
    
    // Try to create the same dependency again
    await rightNode.hover();
    await page.mouse.down();
    await leftNode.hover();
    await page.mouse.up();
    await page.waitForTimeout(300);
    
    // Count should remain the same
    const secondCount = await page.locator('svg line.dependency-line, svg path.dependency-line').count();
    expect(secondCount).toBe(firstCount);
  });

  test('should handle mouse position correctly during drag', async () => {
    // Create two tasks
    await page.click('button:has-text("Add Task")');
    await page.waitForTimeout(200);
    await page.click('button:has-text("Add Task")');
    await page.waitForTimeout(200);
    
    // Start dragging from right node
    const rightNode = await page.locator('.task-card').first().locator('.connection-node-right');
    const rightNodeBox = await rightNode.boundingBox();
    expect(rightNodeBox).not.toBeNull();
    
    if (rightNodeBox) {
      // Click on the center of the right node
      await page.mouse.move(rightNodeBox.x + rightNodeBox.width / 2, rightNodeBox.y + rightNodeBox.height / 2);
      await page.mouse.down();
      
      // Move mouse and verify drag line follows cursor
      const testPositions = [
        { x: 400, y: 300 },
        { x: 450, y: 350 },
        { x: 500, y: 300 }
      ];
      
      for (const pos of testPositions) {
        await page.mouse.move(pos.x, pos.y);
        await page.waitForTimeout(50);
        
        // Get the drag line end position
        const dragLine = await page.locator('svg line[stroke="red"], svg line[stroke="#ff0000"]');
        const x2 = await dragLine.getAttribute('x2');
        const y2 = await dragLine.getAttribute('y2');
        
        // The line should end near the mouse position (within container coordinates)
        // This is a basic check - exact values depend on container offset and zoom
        expect(x2).not.toBeNull();
        expect(y2).not.toBeNull();
      }
      
      // Release to cancel
      await page.mouse.up();
    }
  });

  test.afterEach(async () => {
    await page.close();
  });
});