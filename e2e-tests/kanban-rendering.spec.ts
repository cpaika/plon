import { test, expect, Page } from '@playwright/test';
import { spawn, ChildProcess } from 'child_process';
import { setTimeout } from 'timers/promises';

test.describe('Kanban View Rendering', () => {
  let page: Page;
  let serverProcess: ChildProcess;

  test.beforeAll(async () => {
    // Start the desktop app
    console.log('Starting plon-desktop...');
    serverProcess = spawn('cargo', ['run', '--bin', 'plon-desktop'], {
      stdio: 'pipe'
    });
    
    // Wait for app to start
    await setTimeout(5000);
  });

  test.afterAll(async () => {
    // Kill the server process
    if (serverProcess) {
      serverProcess.kill();
    }
  });

  test.beforeEach(async ({ page: testPage }) => {
    page = testPage;
    // Navigate to the app - desktop app runs on port 8080 by default
    await page.goto('http://localhost:8080');
    await page.waitForLoadState('networkidle');
  });

  test('Kanban view should render columns and cards', async () => {
    // Click on Kanban tab
    await page.click('button:has-text("Kanban")');
    
    // Wait a moment for the view to switch
    await page.waitForTimeout(500);
    
    // Check that Kanban Board title is visible
    const kanbanTitle = await page.locator('h2:has-text("Kanban Board")').isVisible();
    console.log('Kanban title visible:', kanbanTitle);
    expect(kanbanTitle).toBe(true);
    
    // Check for the instruction text
    const instructions = await page.locator('p:has-text("Drag cards between columns")').isVisible();
    console.log('Instructions visible:', instructions);
    expect(instructions).toBe(true);
    
    // Check that columns are rendered
    const todoColumn = await page.locator('h3:has-text("Todo")').isVisible();
    console.log('Todo column visible:', todoColumn);
    expect(todoColumn).toBe(true);
    
    const inProgressColumn = await page.locator('h3:has-text("In Progress")').isVisible();
    console.log('In Progress column visible:', inProgressColumn);
    expect(inProgressColumn).toBe(true);
    
    const reviewColumn = await page.locator('h3:has-text("Review")').isVisible();
    console.log('Review column visible:', reviewColumn);
    expect(reviewColumn).toBe(true);
    
    const doneColumn = await page.locator('h3:has-text("Done")').isVisible();
    console.log('Done column visible:', doneColumn);
    expect(doneColumn).toBe(true);
    
    const blockedColumn = await page.locator('h3:has-text("Blocked")').isVisible();
    console.log('Blocked column visible:', blockedColumn);
    expect(blockedColumn).toBe(true);
    
    // Check that cards are rendered
    const cards = await page.locator('h4').count();
    console.log('Number of cards found:', cards);
    expect(cards).toBeGreaterThan(0);
    
    // Check for specific card titles
    const designCard = await page.locator('h4:has-text("Design dashboard layout")').isVisible();
    console.log('Design card visible:', designCard);
    expect(designCard).toBe(true);
    
    const dataModelsCard = await page.locator('h4:has-text("Implement data models")').isVisible();
    console.log('Data models card visible:', dataModelsCard);
    expect(dataModelsCard).toBe(true);
    
    // Take a screenshot for debugging
    await page.screenshot({ path: 'test-results/kanban-view.png', fullPage: true });
    
    // Check what's actually visible on the page
    const pageContent = await page.content();
    console.log('Page contains "Ready":', pageContent.includes('Ready'));
    console.log('Page contains "Kanban Board":', pageContent.includes('Kanban Board'));
    
    // Get all visible text
    const visibleText = await page.evaluate(() => document.body.innerText);
    console.log('Visible text on page:', visibleText);
  });

  test('Check if Kanban content is being hidden by CSS', async () => {
    // Click on Kanban tab
    await page.click('button:has-text("Kanban")');
    await page.waitForTimeout(500);
    
    // Check main content area
    const mainContent = await page.locator('.main-content').boundingBox();
    console.log('Main content dimensions:', mainContent);
    
    // Check if there's any overflow or height issues
    const kanbanContainer = await page.evaluate(() => {
      const elem = document.querySelector('div[style*="padding: 20px"]');
      if (elem) {
        const styles = window.getComputedStyle(elem);
        return {
          display: styles.display,
          visibility: styles.visibility,
          height: styles.height,
          overflow: styles.overflow,
          position: styles.position
        };
      }
      return null;
    });
    console.log('Kanban container styles:', kanbanContainer);
    
    // Check if the component is actually being rendered
    const htmlContent = await page.innerHTML('.main-content');
    console.log('Main content HTML:', htmlContent.substring(0, 500));
  });
});