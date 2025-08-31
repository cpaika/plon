// Basic test script to verify card ordering works
const puppeteer = require('puppeteer');

(async () => {
    console.log('Starting browser...');
    const browser = await puppeteer.launch({ 
        headless: false,
        defaultViewport: null,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    const page = await browser.newPage();
    
    console.log('Navigating to app...');
    await page.goto('http://localhost:8080', { waitUntil: 'networkidle0' });
    
    console.log('Waiting for app to load...');
    await page.waitForTimeout(2000);
    
    console.log('Clicking Kanban button...');
    await page.evaluate(() => {
        const buttons = Array.from(document.querySelectorAll('button'));
        const kanbanBtn = buttons.find(btn => btn.textContent.includes('Kanban'));
        if (kanbanBtn) {
            kanbanBtn.click();
            console.log('Clicked Kanban button');
        } else {
            console.log('Kanban button not found');
        }
    });
    
    await page.waitForTimeout(2000);
    
    console.log('Looking for kanban board...');
    const hasKanban = await page.evaluate(() => {
        const headers = Array.from(document.querySelectorAll('h2, h3'));
        const kanbanHeader = headers.find(h => h.textContent.includes('Kanban'));
        if (kanbanHeader) {
            console.log('Found Kanban header:', kanbanHeader.textContent);
            return true;
        }
        return false;
    });
    
    if (hasKanban) {
        console.log('✓ Kanban board loaded successfully');
        
        // Try to find and drag a card
        console.log('Looking for cards to drag...');
        const cardInfo = await page.evaluate(() => {
            const cards = Array.from(document.querySelectorAll('div'));
            const taskCards = cards.filter(div => {
                const text = div.textContent || '';
                return text.includes('Design dashboard') || 
                       text.includes('Implement data') ||
                       text.includes('API documentation');
            });
            
            console.log(`Found ${taskCards.length} task cards`);
            return taskCards.length;
        });
        
        console.log(`Found ${cardInfo} cards on the board`);
        
        // Try dragging a card
        console.log('Attempting to drag a card...');
        await page.evaluate(() => {
            const cards = Array.from(document.querySelectorAll('div'));
            const card = cards.find(div => div.textContent?.includes('Design dashboard'));
            
            if (card) {
                // Simulate drag start
                const rect = card.getBoundingClientRect();
                const mouseDownEvent = new MouseEvent('mousedown', {
                    bubbles: true,
                    cancelable: true,
                    clientX: rect.left + rect.width / 2,
                    clientY: rect.top + rect.height / 2
                });
                card.dispatchEvent(mouseDownEvent);
                console.log('Mouse down on card');
                
                // Simulate drag move
                const mouseMoveEvent = new MouseEvent('mousemove', {
                    bubbles: true,
                    cancelable: true,
                    clientX: rect.left + rect.width / 2 + 100,
                    clientY: rect.top + rect.height / 2
                });
                document.dispatchEvent(mouseMoveEvent);
                console.log('Mouse moved');
                
                // Check if dragging visual appears
                setTimeout(() => {
                    const fixedElements = Array.from(document.querySelectorAll('div[style*="position: fixed"]'));
                    console.log(`Found ${fixedElements.length} fixed position elements (potential drag ghosts)`);
                }, 100);
            }
        });
        
        await page.waitForTimeout(2000);
        
    } else {
        console.log('✗ Failed to load Kanban board');
    }
    
    console.log('\nTest complete. Browser will close in 5 seconds...');
    await page.waitForTimeout(5000);
    
    await browser.close();
})().catch(console.error);