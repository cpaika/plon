import { test, expect } from '@playwright/test';

test.describe('Quick Bug Detection', () => {
  test('Kanban: Check if drag changes opacity', async ({ page }) => {
    await page.goto('http://localhost:8080/kanban');
    await page.waitForLoadState('networkidle');
    
    const card = page.locator('div[draggable="true"]').first();
    
    // Check initial style
    const initialStyle = await card.getAttribute('style');
    console.log('Initial card style:', initialStyle);
    
    // Start drag
    const bbox = await card.boundingBox();
    if (bbox) {
      await page.mouse.move(bbox.x + bbox.width / 2, bbox.y + bbox.height / 2);
      await page.mouse.down();
      
      // Check style during drag
      const dragStyle = await card.getAttribute('style');
      console.log('Card style during drag:', dragStyle);
      
      // Note: Current implementation doesn't change opacity during drag
      expect(dragStyle).toBe(initialStyle); // This documents current behavior
      
      await page.mouse.up();
    }
  });

  test('Map: Check actual position after drag', async ({ page }) => {
    await page.goto('http://localhost:8080/map');
    await page.waitForLoadState('networkidle');
    
    const card = page.locator('div[draggable="true"]').first();
    
    // Get initial position
    const initialStyle = await card.getAttribute('style') || '';
    const initialLeft = parseFloat(initialStyle.match(/left:\s*([0-9.]+)px/)?.[1] || '0');
    const initialTop = parseFloat(initialStyle.match(/top:\s*([0-9.]+)px/)?.[1] || '0');
    
    console.log(`Initial position: left=${initialLeft}, top=${initialTop}`);
    
    // Perform drag
    const bbox = await card.boundingBox();
    if (bbox) {
      await page.mouse.move(bbox.x + bbox.width / 2, bbox.y + bbox.height / 2);
      await page.mouse.down();
      
      // Drag 200px right and 200px down
      const targetX = bbox.x + bbox.width / 2 + 200;
      const targetY = bbox.y + bbox.height / 2 + 200;
      
      await page.mouse.move(targetX, targetY);
      await page.mouse.up();
      
      await page.waitForTimeout(500);
      
      // Get new position
      const newStyle = await card.getAttribute('style') || '';
      const newLeft = parseFloat(newStyle.match(/left:\s*([0-9.]+)px/)?.[1] || '0');
      const newTop = parseFloat(newStyle.match(/top:\s*([0-9.]+)px/)?.[1] || '0');
      
      console.log(`New position: left=${newLeft}, top=${newTop}`);
      console.log(`Expected approximately: left=${initialLeft + 200}, top=${initialTop + 200}`);
      console.log(`Actual change: left=${newLeft - initialLeft}, top=${newTop - initialTop}`);
      
      // Current implementation moves by fixed 50px
      expect(newLeft - initialLeft).toBe(50);
      expect(newTop - initialTop).toBe(50);
    }
  });

  test('Kanban: Check column highlight on drag over', async ({ page }) => {
    await page.goto('http://localhost:8080/kanban');
    await page.waitForLoadState('networkidle');
    
    const card = page.locator('div[draggable="true"]').first();
    const inProgressColumn = page.locator('div:has(h3:has-text("In Progress"))').first();
    
    // Get initial column background
    const initialBg = await inProgressColumn.evaluate(el => 
      window.getComputedStyle(el).backgroundColor
    );
    console.log('Initial column background:', initialBg);
    
    // Start drag and hover over column
    const cardBbox = await card.boundingBox();
    const columnBbox = await inProgressColumn.boundingBox();
    
    if (cardBbox && columnBbox) {
      await page.mouse.move(cardBbox.x + cardBbox.width / 2, cardBbox.y + cardBbox.height / 2);
      await page.mouse.down();
      
      await page.mouse.move(
        columnBbox.x + columnBbox.width / 2,
        columnBbox.y + columnBbox.height / 2
      );
      
      await page.waitForTimeout(200);
      
      // Check column background during hover
      const hoverBg = await inProgressColumn.evaluate(el => 
        window.getComputedStyle(el).backgroundColor
      );
      console.log('Column background during drag hover:', hoverBg);
      
      // Should change to greenish (#e8f5e9 = rgb(232, 245, 233))
      expect(hoverBg).toBe('rgb(232, 245, 233)');
      
      await page.mouse.up();
    }
  });
});