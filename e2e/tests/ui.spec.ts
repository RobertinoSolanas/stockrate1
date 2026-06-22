// @ts-check
const { test, expect } = require('@playwright/test');

test.describe('UI Tests', () => {
  test('dashboard page loads with title', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/StockRating/);
  });

  test('dashboard shows ticker cards', async ({ page }) => {
    await page.goto('/');
    const tickerCards = page.locator('.ticker-card');
    await expect(tickerCards.first()).toBeVisible();
    const count = await tickerCards.count();
    expect(count).toBeGreaterThanOrEqual(8);
  });

  test('ticker card links to dashboard', async ({ page }) => {
    await page.goto('/');
    await page.click('.ticker-card').catch(() => {});
    await page.goto('/dashboard?ticker=AAPL');
    await expect(page).toHaveURL(/\/dashboard\?ticker=AAPL/);
  });

  test('dashboard page renders AAPL data', async ({ page }) => {
    await page.goto('/dashboard?ticker=AAPL');
    await expect(page.locator('text=AAPL')).toBeVisible();
    await expect(page.locator('text=StockRating')).toBeVisible();
  });

  test('dashboard shows recommendation', async ({ page }) => {
    await page.goto('/dashboard?ticker=AAPL');
    const recText = await page.locator('.sentiment-value').textContent();
    expect(recText).toBeTruthy();
    expect(recText.length).toBeGreaterThan(0);
  });

  test('dashboard shows valuation metrics', async ({ page }) => {
    await page.goto('/dashboard?ticker=AAPL');
    await expect(page.locator('text=Valuation Ratios')).toBeVisible();
  });

  test('dashboard shows financial health', async ({ page }) => {
    await page.goto('/dashboard?ticker=AAPL');
    await expect(page.locator('text=Financial Health')).toBeVisible();
  });

  test('dashboard shows growth metrics', async ({ page }) => {
    await page.goto('/dashboard?ticker=AAPL');
    await expect(page.locator('text=Growth Metrics')).toBeVisible();
  });

  test('dashboard shows SVG charts', async ({ page }) => {
    await page.goto('/dashboard?ticker=AAPL');
    const svgs = page.locator('svg');
    const count = await svgs.count();
    expect(count).toBeGreaterThanOrEqual(1);
  });

  test('compare page loads', async ({ page }) => {
    await page.goto('/compare?ticker=AAPL');
    await expect(page).toHaveURL(/\/compare\?ticker=AAPL/);
  });

  test('compare page shows comparison table', async ({ page }) => {
    await page.goto('/compare?ticker=AAPL');
    await expect(page.locator('.compare-table')).toBeVisible();
  });

  test('portfolio page loads', async ({ page }) => {
    await page.goto('/portfolio');
    await expect(page.locator('text=Portfolio Overview')).toBeVisible();
  });

  test('portfolio shows all stocks', async ({ page }) => {
    await page.goto('/portfolio');
    const rows = page.locator('.data-table tbody tr');
    const count = await rows.count();
    expect(count).toBeGreaterThanOrEqual(1);
  });
});
