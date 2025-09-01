import { Page, Locator, expect } from '@playwright/test';

export class TestHelpers {
  constructor(private page: Page) {}

  async navigateToKanban() {
    await this.page.goto('/kanban');
    await this.page.waitForSelector('h2:has-text("Kanban Board")');
  }

  async navigateToMap() {
    await this.page.goto('/map');
    await this.page.waitForSelector('h2:has-text("Task Map")');
  }

  async getKanbanColumn(columnName: string): Promise<Locator> {
    return this.page.locator(`div:has(h3:has-text("${columnName}"))`).first();
  }

  async getTaskCardsInColumn(columnName: string): Promise<Locator> {
    const column = await this.getKanbanColumn(columnName);
    return column.locator('div[draggable="true"]');
  }

  async getTaskCardByTitle(title: string): Promise<Locator> {
    return this.page.locator(`div[draggable="true"]:has(h4:has-text("${title}"))`).first();
  }

  async getColumnTaskCount(columnName: string): Promise<number> {
    const column = await this.getKanbanColumn(columnName);
    const countElement = column.locator('span').first();
    const text = await countElement.textContent();
    return parseInt(text || '0', 10);
  }

  async dragAndDrop(source: Locator, target: Locator) {
    // Get bounding boxes
    const sourceBbox = await source.boundingBox();
    const targetBbox = await target.boundingBox();
    
    if (!sourceBbox || !targetBbox) {
      throw new Error('Could not get bounding boxes for drag and drop');
    }

    // Start drag from center of source
    await this.page.mouse.move(
      sourceBbox.x + sourceBbox.width / 2,
      sourceBbox.y + sourceBbox.height / 2
    );
    await this.page.mouse.down();

    // Move to target center
    await this.page.mouse.move(
      targetBbox.x + targetBbox.width / 2,
      targetBbox.y + targetBbox.height / 2,
      { steps: 10 }
    );

    // Drop
    await this.page.mouse.up();
    
    // Wait for DOM to update
    await this.page.waitForTimeout(500);
  }

  async getMapTaskCards(): Promise<Locator> {
    return this.page.locator('div[draggable="true"]');
  }

  async getTaskPosition(taskTitle: string): Promise<{ x: number; y: number }> {
    const task = await this.getTaskCardByTitle(taskTitle);
    const style = await task.getAttribute('style');
    
    if (!style) {
      throw new Error(`Could not get style for task: ${taskTitle}`);
    }

    // Parse position from style attribute
    const leftMatch = style.match(/left:\s*([0-9.]+)px/);
    const topMatch = style.match(/top:\s*([0-9.]+)px/);
    
    if (!leftMatch || !topMatch) {
      throw new Error(`Could not parse position from style: ${style}`);
    }

    return {
      x: parseFloat(leftMatch[1]),
      y: parseFloat(topMatch[1])
    };
  }

  async clickZoomButton(buttonText: string) {
    await this.page.locator(`button:has-text("${buttonText}")`).click();
    await this.page.waitForTimeout(300);
  }

  async getZoomLevel(): Promise<number> {
    const zoomText = await this.page.locator('span:has-text("Zoom:")').textContent();
    if (!zoomText) return 100;
    
    const match = zoomText.match(/(\d+)%/);
    return match ? parseInt(match[1], 10) : 100;
  }

  async waitForDragFeedback(column: Locator, shouldBeHighlighted: boolean) {
    const expectedBackground = shouldBeHighlighted ? '#e8f5e9' : 'white';
    await expect(column).toHaveCSS('background-color', expectedBackground, { timeout: 2000 });
  }

  async takeDebugScreenshot(name: string) {
    await this.page.screenshot({ 
      path: `e2e-tests/screenshots/${name}.png`,
      fullPage: true 
    });
  }
}

export async function setupTest(page: Page): Promise<TestHelpers> {
  // Set viewport for consistent testing
  await page.setViewportSize({ width: 1280, height: 720 });
  
  // Navigate to home page first
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  
  return new TestHelpers(page);
}