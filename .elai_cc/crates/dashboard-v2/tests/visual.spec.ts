import { test, expect } from '@playwright/test';

test.describe('Dashboard Visual Regression', () => {
  test('workspace screen with drawers', async ({ page }) => {
    // Navigate to the dashboard
    await page.goto('/');

    // Wait for the app to load (check for app root)
    await page.waitForSelector('.app-root', { timeout: 30000 });

    // Wait for fonts to load and layout to stabilize
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // Take a full-page screenshot of the workspace
    await expect(page).toHaveScreenshot('workspace-full.png', {
      fullPage: true,
      maxDiffPixels: 100, // Allow for minor rendering differences
    });
  });

  test('workspace screen - viewport only', async ({ page }) => {
    await page.goto('/');

    // Wait for the app to load
    await page.waitForSelector('.app-root', { timeout: 30000 });
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // Take a viewport-only screenshot
    await expect(page).toHaveScreenshot('workspace-viewport.png', {
      maxDiffPixels: 100,
    });
  });

  test('color tokens verification', async ({ page }) => {
    await page.goto('/');

    await page.waitForSelector('.app-root', { timeout: 30000 });
    await page.waitForLoadState('networkidle');

    // Verify background color matches spec (#0a0e12)
    const bgColor = await page.evaluate(() => {
      const styles = window.getComputedStyle(document.body);
      return styles.backgroundColor;
    });

    // Convert RGB to hex for comparison
    const rgbToHex = (rgb: string) => {
      const match = rgb.match(/^rgb\((\d+),\s*(\d+),\s*(\d+)\)$/);
      if (!match) return null;
      const [, r, g, b] = match;
      return `#${parseInt(r).toString(16).padStart(2, '0')}${parseInt(g).toString(16).padStart(2, '0')}${parseInt(b).toString(16).padStart(2, '0')}`;
    };

    const hexColor = rgbToHex(bgColor);
    expect(hexColor).toBe('#0a0e12');
  });

  test('font family verification', async ({ page }) => {
    await page.goto('/');

    await page.waitForSelector('.app-root', { timeout: 30000 });
    await page.waitForLoadState('networkidle');

    // Verify JetBrains Mono is loaded
    const fontFamily = await page.evaluate(() => {
      const styles = window.getComputedStyle(document.body);
      return styles.fontFamily;
    });

    expect(fontFamily).toContain('JetBrains Mono');
    expect(fontFamily).not.toContain('Inter');
  });
});
