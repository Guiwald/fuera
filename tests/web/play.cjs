(async () => {
// Plays the web build in Chromium: home, start level 1, swipe, screenshots.
// Usage: node tests/web/play.mjs http://localhost:8123/ outdir
const { chromium } = require('playwright');
const [url, out] = process.argv.slice(2);
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 412, height: 915 }, deviceScaleFactor: 2, hasTouch: false });
const logs = [];
page.on('console', m => logs.push(`[${m.type()}] ${m.text()}`));
page.on('pageerror', e => logs.push(`[pageerror] ${e.message}`));
await page.goto(url);
await page.waitForSelector('canvas', { timeout: 15000 });
await page.waitForTimeout(1200);
await page.screenshot({ path: `${out}/web-1-home.png` });
// Tap "Play" (primary button around 60% height).
await page.mouse.click(206, 915 * 0.58 + 34);
await page.waitForTimeout(800);
await page.screenshot({ path: `${out}/web-2-level.png` });
// Swipe: find the canvas and drag from the first block to the right.
const drag = async (x0, y0, x1, y1) => {
  await page.mouse.move(x0, y0); await page.mouse.down();
  for (let i = 1; i <= 6; i++) { await page.mouse.move(x0 + (x1 - x0) * i / 6, y0 + (y1 - y0) * i / 6); await page.waitForTimeout(16); }
  await page.mouse.up();
};
const args = process.argv.slice(4).map(Number);
if (args.length >= 4) { await drag(args[0], args[1], args[2], args[3]); }
await page.waitForTimeout(900);
await page.screenshot({ path: `${out}/web-3-after.png` });
console.log(logs.join('\n'));
await browser.close();

})();
