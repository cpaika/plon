import { test, expect, Page } from '@playwright/test';

test.describe('Map View Dependency Creation', () => {
  let page: Page;

  test.beforeEach(async ({ browser }) => {
    page = await browser.newPage();
    await page.goto('http://localhost:8080');
    
    // Navigate to Map view
    await page.click('button:has-text("Map")');
    await page.waitForSelector('h2:has-text("Task Map")');
  });

  test.afterEach(async () => {
    await page.close();
  });

  test('Can enter dependency creation mode', async () => {
    // Look for dependency mode button
    await page.click('button:has-text("Create Dependency")');
    
    // Verify mode is active
    const modeIndicator = await page.locator('text=Creating dependency').isVisible();
    expect(modeIndicator).toBe(true);
    
    // Should be able to exit mode
    await page.keyboard.press('Escape');
    const modeGone = await page.locator('text=Creating dependency').isHidden();
    expect(modeGone).toBe(true);
  });

  test('Can create dependency between two tasks', async () => {
    // Enter dependency creation mode
    await page.click('button:has-text("Create Dependency")');
    
    // Get initial task positions for visual verification
    const task1 = await page.locator('div:has-text("Design dashboard")').first();
    const task2 = await page.locator('div:has-text("Implement data")').first();
    
    // Click first task (source)
    await task1.click();
    
    // Should show visual feedback that first task is selected
    const task1Selected = await task1.evaluate(el => 
      window.getComputedStyle(el).borderColor.includes('255') || 
      window.getComputedStyle(el).borderColor.includes('4CAF50')
    );
    expect(task1Selected).toBe(true);
    
    // Click second task (target)
    await task2.click();
    
    // Dependency should be created - look for visual line/arrow
    await page.waitForTimeout(500);
    
    // Check for SVG line or path element connecting the tasks
    const hasDependencyLine = await page.evaluate(() => {
      const svgs = document.querySelectorAll('svg');
      for (const svg of svgs) {
        const lines = svg.querySelectorAll('line, path');
        if (lines.length > 0) return true;
      }
      return false;
    });
    expect(hasDependencyLine).toBe(true);
    
    // Mode should exit after creating dependency
    const modeExited = await page.locator('text=Creating dependency').isHidden();
    expect(modeExited).toBe(true);
  });

  test('Shows visual feedback during dependency creation', async () => {
    await page.click('button:has-text("Create Dependency")');
    
    const task1 = await page.locator('div:has-text("Design dashboard")').first();
    
    // Hover should show visual feedback
    await task1.hover();
    const hasHoverEffect = await task1.evaluate(el => {
      const style = window.getComputedStyle(el);
      return style.cursor === 'crosshair' || style.cursor === 'pointer';
    });
    expect(hasHoverEffect).toBe(true);
    
    // Click first task
    await task1.click();
    
    // Moving mouse should show line preview
    const task2Box = await page.locator('div:has-text("Implement data")').first().boundingBox();
    if (task2Box) {
      await page.mouse.move(task2Box.x + task2Box.width / 2, task2Box.y + task2Box.height / 2);
      
      // Check for preview line
      const hasPreviewLine = await page.evaluate(() => {
        const elements = document.querySelectorAll('[style*="dashed"], [stroke-dasharray]');
        return elements.length > 0;
      });
      expect(hasPreviewLine).toBe(true);
    }
  });

  test('Prevents circular dependencies', async () => {
    // First create a dependency from Task A to Task B
    await page.click('button:has-text("Create Dependency")');
    
    const taskA = await page.locator('div:has-text("Design dashboard")').first();
    const taskB = await page.locator('div:has-text("Implement data")').first();
    
    await taskA.click();
    await taskB.click();
    
    // Try to create reverse dependency (should fail)
    await page.click('button:has-text("Create Dependency")');
    await taskB.click();
    await taskA.click();
    
    // Should show error message
    const hasError = await page.locator('text=/cycle|circular/i').isVisible();
    expect(hasError).toBe(true);
  });

  test('Can delete dependencies', async () => {
    // Create a dependency first
    await page.click('button:has-text("Create Dependency")');
    const task1 = await page.locator('div:has-text("Design dashboard")').first();
    const task2 = await page.locator('div:has-text("Implement data")').first();
    
    await task1.click();
    await task2.click();
    
    // Right-click on dependency line to delete (or use delete mode)
    await page.waitForTimeout(500);
    
    // Find and click on the dependency line
    const dependencyDeleted = await page.evaluate(() => {
      const svgs = document.querySelectorAll('svg');
      for (const svg of svgs) {
        const lines = svg.querySelectorAll('line, path');
        for (const line of lines) {
          // Simulate right-click on line
          const evt = new MouseEvent('contextmenu', {
            bubbles: true,
            cancelable: true,
            view: window
          });
          line.dispatchEvent(evt);
          return true;
        }
      }
      return false;
    });
    
    if (dependencyDeleted) {
      // Look for delete option in context menu
      const deleteOption = await page.locator('text=Delete dependency');
      if (await deleteOption.isVisible()) {
        await deleteOption.click();
        
        // Verify line is gone
        await page.waitForTimeout(500);
        const hasNoLines = await page.evaluate(() => {
          const svgs = document.querySelectorAll('svg');
          for (const svg of svgs) {
            const lines = svg.querySelectorAll('line, path');
            if (lines.length > 0) return false;
          }
          return true;
        });
        expect(hasNoLines).toBe(true);
      }
    }
  });

  test('Dependencies persist after page refresh', async () => {
    // Create a dependency
    await page.click('button:has-text("Create Dependency")');
    
    const task1 = await page.locator('div:has-text("Design dashboard")').first();
    const task2 = await page.locator('div:has-text("Implement data")').first();
    
    await task1.click();
    await task2.click();
    
    await page.waitForTimeout(500);
    
    // Count dependencies before refresh
    const dependenciesBeforeRefresh = await page.evaluate(() => {
      let count = 0;
      const svgs = document.querySelectorAll('svg');
      for (const svg of svgs) {
        count += svg.querySelectorAll('line, path').length;
      }
      return count;
    });
    
    // Refresh page
    await page.reload();
    await page.waitForSelector('h2:has-text("Task Map")');
    await page.waitForTimeout(1000);
    
    // Count dependencies after refresh
    const dependenciesAfterRefresh = await page.evaluate(() => {
      let count = 0;
      const svgs = document.querySelectorAll('svg');
      for (const svg of svgs) {
        count += svg.querySelectorAll('line, path').length;
      }
      return count;
    });
    
    expect(dependenciesAfterRefresh).toBe(dependenciesBeforeRefresh);
    expect(dependenciesAfterRefresh).toBeGreaterThan(0);
  });

  test('Shows dependency arrows with correct direction', async () => {
    await page.click('button:has-text("Create Dependency")');
    
    const task1 = await page.locator('div:has-text("Design dashboard")').first();
    const task2 = await page.locator('div:has-text("Implement data")').first();
    
    await task1.click();
    await task2.click();
    
    await page.waitForTimeout(500);
    
    // Check for arrowhead marker
    const hasArrowhead = await page.evaluate(() => {
      const markers = document.querySelectorAll('marker, [marker-end]');
      return markers.length > 0;
    });
    expect(hasArrowhead).toBe(true);
  });

  test('Highlights connected tasks when hovering dependency', async () => {
    // Create a dependency
    await page.click('button:has-text("Create Dependency")');
    
    const task1 = await page.locator('div:has-text("Design dashboard")').first();
    const task2 = await page.locator('div:has-text("Implement data")').first();
    
    await task1.click();
    await task2.click();
    
    await page.waitForTimeout(500);
    
    // Hover over the dependency line
    const svgLine = await page.locator('svg line, svg path').first();
    await svgLine.hover();
    
    // Check if connected tasks are highlighted
    const task1Highlighted = await task1.evaluate(el => {
      const style = window.getComputedStyle(el);
      return style.boxShadow.includes('0 4px') || style.transform.includes('scale');
    });
    
    const task2Highlighted = await task2.evaluate(el => {
      const style = window.getComputedStyle(el);
      return style.boxShadow.includes('0 4px') || style.transform.includes('scale');
    });
    
    expect(task1Highlighted || task2Highlighted).toBe(true);
  });
});