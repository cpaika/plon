import { test, expect } from '@playwright/test';

test.describe('Kanban View', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/kanban');
  });

  test('should display all task columns', async ({ page }) => {
    // Check that all columns are visible
    await expect(page.locator('.kanban-column').nth(0)).toContainText('To Do');
    await expect(page.locator('.kanban-column').nth(1)).toContainText('In Progress');
    await expect(page.locator('.kanban-column').nth(2)).toContainText('Done');
    await expect(page.locator('.kanban-column').nth(3)).toContainText('Blocked');
  });

  test('should drag and drop task between columns', async ({ page }) => {
    // Create a test task
    const taskTitle = `Test Task ${Date.now()}`;
    await page.click('.kanban-column:first-child .add-task-btn');
    await page.fill('input[placeholder="Task title"]', taskTitle);
    await page.press('input[placeholder="Task title"]', 'Enter');

    // Find the task card
    const taskCard = page.locator('.kanban-card', { hasText: taskTitle });
    await expect(taskCard).toBeVisible();

    // Drag to In Progress column
    const inProgressColumn = page.locator('.kanban-column', { hasText: 'In Progress' });
    await taskCard.dragTo(inProgressColumn);

    // Verify task moved
    await expect(inProgressColumn.locator('.kanban-card', { hasText: taskTitle })).toBeVisible();
  });

  test('should show play button on Todo tasks', async ({ page }) => {
    // Find a Todo task
    const todoColumn = page.locator('.kanban-column', { hasText: 'To Do' });
    const todoCard = todoColumn.locator('.kanban-card').first();
    
    // Check for play button
    await expect(todoCard.locator('.btn-icon', { hasText: '▶️' })).toBeVisible();
  });

  test('should start Claude Code execution when play button clicked', async ({ page }) => {
    // Find a Todo task with play button
    const todoColumn = page.locator('.kanban-column', { hasText: 'To Do' });
    const playButton = todoColumn.locator('.btn-icon', { hasText: '▶️' }).first();
    
    // Click play button
    await playButton.click();
    
    // Verify running indicator appears
    await expect(page.locator('.running-indicator')).toBeVisible();
  });

  test('should select task on click', async ({ page }) => {
    // Click on a task card
    const taskCard = page.locator('.kanban-card').first();
    await taskCard.click();
    
    // Verify task is selected
    await expect(taskCard).toHaveClass(/selected/);
  });

  test('should filter tasks by status', async ({ page }) => {
    // This test would apply if filter controls are added to Kanban view
    // For now, verify all status columns are present
    const columns = await page.locator('.kanban-column').count();
    expect(columns).toBe(4);
  });

  test('should handle empty columns gracefully', async ({ page }) => {
    // Check that empty columns still show add button
    const columns = page.locator('.kanban-column');
    const columnCount = await columns.count();
    
    for (let i = 0; i < columnCount; i++) {
      const column = columns.nth(i);
      await expect(column.locator('.add-task-btn')).toBeVisible();
    }
  });
});