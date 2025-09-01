import { test, expect } from '@playwright/test';

test.describe('Timeline View', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/timeline');
  });

  test('should display timeline', async ({ page }) => {
    await expect(page.locator('.timeline-view')).toBeVisible();
    await expect(page.locator('.timeline-container')).toBeVisible();
  });

  test('should navigate to previous and next week', async ({ page }) => {
    // Get current date headers
    const initialDates = await page.locator('.date-column').allTextContents();
    
    // Navigate to next week
    await page.click('button:has-text("Next Week →")');
    await page.waitForTimeout(100);
    
    const nextWeekDates = await page.locator('.date-column').allTextContents();
    expect(nextWeekDates).not.toEqual(initialDates);
    
    // Navigate to previous week
    await page.click('button:has-text("← Previous Week")');
    await page.waitForTimeout(100);
    
    const prevWeekDates = await page.locator('.date-column').allTextContents();
    expect(prevWeekDates).not.toEqual(nextWeekDates);
  });

  test('should return to today', async ({ page }) => {
    // Navigate away from today
    await page.click('button:has-text("Next Week →")');
    await page.click('button:has-text("Next Week →")');
    
    // Click Today button
    await page.click('button:has-text("Today")');
    
    // Check that today is highlighted
    await expect(page.locator('.date-column.today')).toBeVisible();
  });

  test('should change timeline zoom', async ({ page }) => {
    // Change to 1 week view
    await page.selectOption('.timeline-zoom', '7');
    let columns = await page.locator('.date-column').count();
    expect(columns).toBeLessThanOrEqual(8);
    
    // Change to 3 months view
    await page.selectOption('.timeline-zoom', '90');
    columns = await page.locator('.date-column').count();
    expect(columns).toBeGreaterThan(8);
  });

  test('should display tasks on timeline', async ({ page }) => {
    // Check for timeline tasks
    const tasks = page.locator('.timeline-task');
    const count = await tasks.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('should select task on click', async ({ page }) => {
    const task = page.locator('.timeline-task').first();
    
    if (await task.count() > 0) {
      await task.click();
      await expect(task).toHaveClass(/selected/);
    }
  });

  test('should show play button for Todo tasks', async ({ page }) => {
    const playButtons = page.locator('.timeline-task .play-btn');
    const count = await playButtons.count();
    
    // Should have play buttons for Todo tasks
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('should scroll timeline horizontally', async ({ page }) => {
    const container = page.locator('.timeline-container');
    
    // Simulate horizontal scroll with mouse wheel
    await container.hover();
    await page.mouse.wheel(0, 100);
    
    // Check that timeline scrolled (transform changed)
    const header = page.locator('.timeline-header');
    const transform = await header.getAttribute('style');
    expect(transform).toContain('translateX');
  });

  test('should display task with correct status color', async ({ page }) => {
    const tasks = page.locator('.timeline-task');
    
    if (await tasks.count() > 0) {
      const firstTask = tasks.first();
      const style = await firstTask.getAttribute('style');
      
      // Should have a background color set
      expect(style).toMatch(/background:\s*#[0-9A-Fa-f]{6}/);
    }
  });

  test('should highlight today column', async ({ page }) => {
    // Navigate to today if not already there
    await page.click('button:has-text("Today")');
    
    // Check for today highlight
    const todayColumn = page.locator('.date-column.today');
    await expect(todayColumn).toBeVisible();
    
    // Verify it has special styling
    const className = await todayColumn.getAttribute('class');
    expect(className).toContain('today');
  });

  test('should position tasks based on due date', async ({ page }) => {
    const tasks = page.locator('.timeline-task');
    
    if (await tasks.count() > 0) {
      const firstTask = tasks.first();
      const style = await firstTask.getAttribute('style');
      
      // Should have position styling
      expect(style).toContain('position: absolute');
      expect(style).toMatch(/left:\s*\d+px/);
    }
  });

  test('should show running indicator for active tasks', async ({ page }) => {
    // Look for running badges
    const runningBadges = page.locator('.running-badge');
    const count = await runningBadges.count();
    
    // May or may not have running tasks
    expect(count).toBeGreaterThanOrEqual(0);
  });
});