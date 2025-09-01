import { test, expect, Page } from '@playwright/test';

test.describe('Kanban Database Persistence', () => {
  let page: Page;
  
  // Helper function to wait for Kanban to load
  async function waitForKanbanLoad(page: Page) {
    await page.waitForSelector('h2:has-text("Kanban Board")', { timeout: 10000 });
    // Wait for loading to complete
    const loadingVisible = await page.locator('text=Loading...').isVisible();
    if (loadingVisible) {
      await page.waitForSelector('text=Loading...', { state: 'hidden', timeout: 10000 });
    }
    // Wait for columns to be visible
    await page.waitForSelector('h3:has-text("Todo")', { timeout: 5000 });
  }
  
  // Helper to get column task counts
  async function getColumnCounts(page: Page) {
    const counts: Record<string, number> = {};
    
    // Get Todo count
    const todoColumn = page.locator('div:has(> div > h3:has-text("Todo"))').first();
    const todoCount = await todoColumn.locator('span[style*="border-radius: 12px"]').first().textContent();
    counts['todo'] = parseInt(todoCount || '0');
    
    // Get In Progress count
    const inProgressColumn = page.locator('div:has(> div > h3:has-text("In Progress"))').first();
    const inProgressCount = await inProgressColumn.locator('span[style*="border-radius: 12px"]').first().textContent();
    counts['inProgress'] = parseInt(inProgressCount || '0');
    
    // Get Review count
    const reviewColumn = page.locator('div:has(> div > h3:has-text("Review"))').first();
    const reviewCount = await reviewColumn.locator('span[style*="border-radius: 12px"]').first().textContent();
    counts['review'] = parseInt(reviewCount || '0');
    
    // Get Done count
    const doneColumn = page.locator('div:has(> div > h3:has-text("Done"))').first();
    const doneCount = await doneColumn.locator('span[style*="border-radius: 12px"]').first().textContent();
    counts['done'] = parseInt(doneCount || '0');
    
    // Get Blocked count
    const blockedColumn = page.locator('div:has(> div > h3:has-text("Blocked"))').first();
    const blockedCount = await blockedColumn.locator('span[style*="border-radius: 12px"]').first().textContent();
    counts['blocked'] = parseInt(blockedCount || '0');
    
    return counts;
  }

  test.beforeEach(async ({ page: testPage }) => {
    page = testPage;
    // For desktop app testing - it should be running on a local port
    // If testing web version, use port 8080
    await page.goto('http://localhost:8080', { waitUntil: 'networkidle' });
  });

  test('Task moves persist when navigating away and back', async () => {
    // Navigate to Kanban view
    await page.click('button:has-text("Kanban")');
    await waitForKanbanLoad(page);
    
    // Get initial counts
    const initialCounts = await getColumnCounts(page);
    console.log('Initial counts:', initialCounts);
    
    // Find a task in Todo column and drag it to In Progress
    const todoColumn = page.locator('div:has(> div > h3:has-text("Todo"))').first();
    const todoCard = todoColumn.locator('div[style*="cursor"][style*="background: white"]').first();
    
    // Get the task title before moving
    const taskTitle = await todoCard.locator('h4').textContent();
    console.log('Moving task:', taskTitle);
    
    // Perform drag and drop to In Progress
    const inProgressColumn = page.locator('div:has(> div > h3:has-text("In Progress"))').first();
    await todoCard.hover();
    await page.mouse.down();
    await inProgressColumn.hover();
    await page.mouse.up();
    
    // Wait for the move to complete
    await page.waitForTimeout(1000);
    
    // Verify counts changed
    const afterMoveCounts = await getColumnCounts(page);
    console.log('After move counts:', afterMoveCounts);
    expect(afterMoveCounts.todo).toBe(initialCounts.todo - 1);
    expect(afterMoveCounts.inProgress).toBe(initialCounts.inProgress + 1);
    
    // Navigate to a different view
    await page.click('button:has-text("List")');
    await page.waitForSelector('h2:has-text("Task List")', { timeout: 5000 }).catch(() => {
      console.log('List view header not found, continuing...');
    });
    await page.waitForTimeout(500);
    
    // Navigate back to Kanban
    await page.click('button:has-text("Kanban")');
    await waitForKanbanLoad(page);
    
    // Verify the task is still in In Progress
    const finalCounts = await getColumnCounts(page);
    console.log('Final counts after navigation:', finalCounts);
    expect(finalCounts.todo).toBe(afterMoveCounts.todo);
    expect(finalCounts.inProgress).toBe(afterMoveCounts.inProgress);
    
    // Verify the specific task is in In Progress column
    const inProgressCards = await inProgressColumn.locator('h4').allTextContents();
    expect(inProgressCards).toContain(taskTitle);
  });

  test('Multiple task moves persist correctly', async () => {
    // Navigate to Kanban view
    await page.click('button:has-text("Kanban")');
    await waitForKanbanLoad(page);
    
    // Track tasks we're moving
    const movedTasks: { title: string, targetColumn: string }[] = [];
    
    // Move first task from Todo to Review
    const todoColumn = page.locator('div:has(> div > h3:has-text("Todo"))').first();
    let todoCard = todoColumn.locator('div[style*="cursor"][style*="background: white"]').first();
    let taskTitle = await todoCard.locator('h4').textContent();
    movedTasks.push({ title: taskTitle!, targetColumn: 'Review' });
    
    const reviewColumn = page.locator('div:has(> div > h3:has-text("Review"))').first();
    await todoCard.hover();
    await page.mouse.down();
    await reviewColumn.hover();
    await page.mouse.up();
    await page.waitForTimeout(500);
    
    // Move a task from In Progress to Done
    const inProgressColumn = page.locator('div:has(> div > h3:has-text("In Progress"))').first();
    const inProgressCard = inProgressColumn.locator('div[style*="cursor"][style*="background: white"]').first();
    taskTitle = await inProgressCard.locator('h4').textContent();
    movedTasks.push({ title: taskTitle!, targetColumn: 'Done' });
    
    const doneColumn = page.locator('div:has(> div > h3:has-text("Done"))').first();
    await inProgressCard.hover();
    await page.mouse.down();
    await doneColumn.hover();
    await page.mouse.up();
    await page.waitForTimeout(500);
    
    // Move another Todo task to Blocked
    todoCard = todoColumn.locator('div[style*="cursor"][style*="background: white"]').first();
    taskTitle = await todoCard.locator('h4').textContent();
    movedTasks.push({ title: taskTitle!, targetColumn: 'Blocked' });
    
    const blockedColumn = page.locator('div:has(> div > h3:has-text("Blocked"))').first();
    await todoCard.hover();
    await page.mouse.down();
    await blockedColumn.hover();
    await page.mouse.up();
    await page.waitForTimeout(500);
    
    console.log('Moved tasks:', movedTasks);
    
    // Navigate away and back
    await page.click('button:has-text("Map")');
    await page.waitForTimeout(1000);
    await page.click('button:has-text("Kanban")');
    await waitForKanbanLoad(page);
    
    // Verify all moves persisted
    for (const task of movedTasks) {
      const column = page.locator(`div:has(> div > h3:has-text("${task.targetColumn}"))`).first();
      const columnCards = await column.locator('h4').allTextContents();
      expect(columnCards).toContain(task.title);
      console.log(`✓ Task "${task.title}" found in ${task.targetColumn} column`);
    }
  });

  test('Deleted tasks stay deleted after navigation', async () => {
    // Navigate to Kanban view
    await page.click('button:has-text("Kanban")');
    await waitForKanbanLoad(page);
    
    // Get initial total task count
    const initialCards = await page.locator('h4').count();
    console.log('Initial task count:', initialCards);
    
    // Find and delete a task
    const firstCard = page.locator('div[style*="cursor"][style*="background: white"]').first();
    const taskTitle = await firstCard.locator('h4').textContent();
    console.log('Deleting task:', taskTitle);
    
    // Click the delete button (×)
    const deleteButton = firstCard.locator('button:has-text("×")').first();
    await deleteButton.click();
    
    // Wait for deletion
    await page.waitForTimeout(500);
    
    // Verify task count decreased
    const afterDeleteCount = await page.locator('h4').count();
    expect(afterDeleteCount).toBe(initialCards - 1);
    
    // Navigate away and back
    await page.click('button:has-text("Dashboard")');
    await page.waitForTimeout(1000);
    await page.click('button:has-text("Kanban")');
    await waitForKanbanLoad(page);
    
    // Verify the task is still deleted
    const finalCount = await page.locator('h4').count();
    expect(finalCount).toBe(afterDeleteCount);
    
    // Verify the specific task is not present
    const allTitles = await page.locator('h4').allTextContents();
    expect(allTitles).not.toContain(taskTitle);
    console.log(`✓ Task "${taskTitle}" remains deleted after navigation`);
  });

  test('Task state persists across page refresh', async () => {
    // Navigate to Kanban view
    await page.click('button:has-text("Kanban")');
    await waitForKanbanLoad(page);
    
    // Get initial state
    const initialCounts = await getColumnCounts(page);
    
    // Move a task from Todo to Done
    const todoColumn = page.locator('div:has(> div > h3:has-text("Todo"))').first();
    const todoCard = todoColumn.locator('div[style*="cursor"][style*="background: white"]').first();
    const taskTitle = await todoCard.locator('h4').textContent();
    
    const doneColumn = page.locator('div:has(> div > h3:has-text("Done"))').first();
    await todoCard.hover();
    await page.mouse.down();
    await doneColumn.hover();
    await page.mouse.up();
    await page.waitForTimeout(1000);
    
    // Refresh the page
    await page.reload({ waitUntil: 'networkidle' });
    
    // Navigate back to Kanban
    await page.click('button:has-text("Kanban")');
    await waitForKanbanLoad(page);
    
    // Verify the task is still in Done column
    const doneCards = await doneColumn.locator('h4').allTextContents();
    expect(doneCards).toContain(taskTitle);
    
    // Verify counts
    const finalCounts = await getColumnCounts(page);
    expect(finalCounts.todo).toBe(initialCounts.todo - 1);
    expect(finalCounts.done).toBe(initialCounts.done + 1);
    
    console.log(`✓ Task "${taskTitle}" persisted in Done column after page refresh`);
  });
});