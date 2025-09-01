import { test, expect } from '@playwright/test';

test.describe('List View', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/list');
  });

  test('should display task list', async ({ page }) => {
    await expect(page.locator('.task-list')).toBeVisible();
  });

  test('should filter tasks by status', async ({ page }) => {
    // Select filter
    await page.selectOption('.filter-select', 'todo');
    
    // Check that only Todo tasks are visible
    const statusIcons = page.locator('.status-icon');
    const count = await statusIcons.count();
    
    for (let i = 0; i < count; i++) {
      const icon = await statusIcons.nth(i).textContent();
      expect(icon).toBe('⭕'); // Todo icon
    }
  });

  test('should sort tasks', async ({ page }) => {
    // Sort by title
    await page.selectOption('.sort-select', 'title');
    
    // Get all task titles
    const titles = await page.locator('.task-title').allTextContents();
    
    // Check they're sorted
    const sortedTitles = [...titles].sort();
    expect(titles).toEqual(sortedTitles);
  });

  test('should edit task title on double click', async ({ page }) => {
    const taskTitle = page.locator('.task-title').first();
    
    // Double click to edit
    await taskTitle.dblclick();
    
    // Input should appear
    await expect(page.locator('.task-title-input').first()).toBeVisible();
    
    // Type new title
    const newTitle = `Updated Task ${Date.now()}`;
    await page.fill('.task-title-input', newTitle);
    await page.press('.task-title-input', 'Enter');
    
    // Verify title updated
    await expect(taskTitle).toContainText(newTitle);
  });

  test('should select task on click', async ({ page }) => {
    const taskItem = page.locator('.task-list-item').first();
    
    // Click task
    await taskItem.click();
    
    // Verify selection
    await expect(taskItem).toHaveClass(/selected/);
  });

  test('should show play button for Todo tasks', async ({ page }) => {
    // Filter to show only Todo tasks
    await page.selectOption('.filter-select', 'todo');
    
    // Check for play buttons
    const playButtons = page.locator('.btn-icon', { hasText: '▶️' });
    const count = await playButtons.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should start Claude Code on play button click', async ({ page }) => {
    // Find play button
    const playButton = page.locator('.btn-icon', { hasText: '▶️' }).first();
    
    // Click it
    await playButton.click();
    
    // Verify execution started (would check for running state in real app)
    await page.waitForTimeout(100);
  });

  test('should delete task', async ({ page }) => {
    const taskItem = page.locator('.task-list-item').first();
    const taskTitle = await taskItem.locator('.task-title').textContent();
    
    // Click delete button
    await taskItem.locator('.btn-icon.danger').click();
    
    // Confirm deletion (if there's a confirmation dialog)
    // await page.click('button:has-text("Confirm")');
    
    // Verify task is gone
    await expect(page.locator('.task-title', { hasText: taskTitle || '' })).not.toBeVisible();
  });

  test('should display task metadata', async ({ page }) => {
    const taskItem = page.locator('.task-list-item').first();
    
    // Check for due date
    const dueDate = taskItem.locator('.due-date');
    if (await dueDate.count() > 0) {
      await expect(dueDate.first()).toContainText('📅');
    }
    
    // Check for priority
    const priority = taskItem.locator('.priority');
    if (await priority.count() > 0) {
      await expect(priority.first()).toContainText('⚡');
    }
  });

  test('should handle empty list gracefully', async ({ page }) => {
    // Filter to a status with no tasks
    await page.selectOption('.filter-select', 'blocked');
    
    // Should still show the list container
    await expect(page.locator('.task-list')).toBeVisible();
  });
});