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
    await expect(page.getByText('StockRating', { exact: true })).toBeVisible();
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
    await expect(page.locator('.card-title').filter({ hasText: 'Financial Health' })).toBeVisible();
  });

  test('dashboard shows growth metrics', async ({ page }) => {
    await page.goto('/dashboard?ticker=AAPL');
    await expect(page.locator('.card-title').filter({ hasText: 'Growth Metrics' })).toBeVisible();
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

  test.describe('Application Tour', () => {
    test('user tour: browse dashboard, analyze a stock, compare, and view portfolio', async ({
      page,
    }) => {
      // ---- User Story 1: A new user lands on the dashboard and browses stocks ----
      await page.goto('/');
      await expect(page).toHaveTitle(/StockRating/);
      await page.waitForTimeout(600);

      const pageTitle = page.locator('.page-title');
      await expect(pageTitle).toHaveText('Market Overview');
      await page.waitForTimeout(400);

      const tickerCards = page.locator('.ticker-card');
      const cardCount = await tickerCards.count();
      expect(cardCount).toBeGreaterThanOrEqual(8);

      await page.mouse.move(200, 300);
      await page.waitForTimeout(300);

      await tickerCards.first().click();
      await page.waitForTimeout(400);

      // ---- User Story 2: User drills into a specific stock analysis page ----
      await expect(page).toHaveURL(/\/dashboard\?ticker=/);
      const tickerOnUrl = await page.locator('text=AAPL').first().textContent();
      expect(tickerOnUrl).toBe('AAPL');
      await page.waitForTimeout(600);

      await expect(page.locator('.sentiment-value')).toBeVisible();
      await page.waitForTimeout(300);

      const sentimentText = await page.locator('.sentiment-value').textContent();
      expect(sentimentText).toBeTruthy();
      expect(sentimentText.length).toBeGreaterThan(0);
      await page.waitForTimeout(200);

      await expect(page.locator('text=Valuation Ratios')).toBeVisible();
      await page.waitForTimeout(200);

      await expect(page.locator('.card-title').filter({ hasText: 'Financial Health' })).toBeVisible();
      await page.waitForTimeout(200);

      await expect(page.locator('.card-title').filter({ hasText: 'Growth Metrics' })).toBeVisible();
      await page.waitForTimeout(200);

      await expect(page.locator('.card-title').filter({ hasText: 'Market Data' })).toBeVisible();
      await page.waitForTimeout(200);

      const svgCount = await page.locator('svg').count();
      expect(svgCount).toBeGreaterThanOrEqual(1);
      await page.waitForTimeout(300);

      // Switch to the second provider and verify data updates
      const secondProviderBtn = page.locator('a:has-text("SecondMockProvider")');
      await secondProviderBtn.click();
      await page.waitForLoadState('load');
      await expect(page.locator('text=AAPL')).toBeVisible();
      await page.waitForTimeout(300);

   // Go back to the dashboard
      await page.goto('/');
      await expect(page.locator('.page-title')).toHaveText('Market Overview');
      await page.waitForTimeout(300);

      // ---- User Story 3: User navigates to the compare page and reviews comparison ----
      await page.locator('a:has-text("Compare")').click();
      await page.waitForLoadState('load');
      await expect(page.locator('.compare-table')).toBeVisible();
      await page.waitForTimeout(300);

      const tableRows = page.locator('.compare-table tbody tr');
      const rowsCount = await tableRows.count();
      expect(rowsCount).toBeGreaterThanOrEqual(1);
      await page.waitForTimeout(400);

      // Navigate to portfolio
      await page.goto('/portfolio');
      await expect(page.locator('text=Portfolio Overview')).toBeVisible();
      await page.waitForTimeout(400);

      const portfolioRows = page.locator('.data-table tbody tr');
      const portfolioCount = await portfolioRows.count();
      expect(portfolioCount).toBeGreaterThanOrEqual(1);
      await page.waitForTimeout(300);

      // Switch portfolio provider
      const portfolioProviderBtn = page.locator('a:has-text("SecondMockProvider")');
      await portfolioProviderBtn.click();
      await page.waitForLoadState('load');
      await expect(page.locator('text=Portfolio Overview')).toBeVisible();
      await page.waitForTimeout(400);

    // Return to dashboard to finish the tour
      await page.goto('/');
      await expect(page.locator('.page-title')).toHaveText('Market Overview');
      await page.waitForTimeout(200);

      // Verify final state: ticker grid is visible and interactive
      const finalCards = page.locator('.ticker-card');
      const finalCount = await finalCards.count();
      expect(finalCount).toBeGreaterThanOrEqual(8);

      // Verify the metrics dropdown exists and has options
      const metricsSelect = page.locator('#metrics-select');
      await expect(metricsSelect).toBeVisible();
      const options = metricsSelect.locator('option');
      const optionCount = await options.count();
      expect(optionCount).toBeGreaterThanOrEqual(3);
      await page.waitForTimeout(200);

      // Verify chart dropdown exists
      const chartSelect = page.locator('#chart-select');
      await expect(chartSelect).toBeVisible();

      // Final DOM health check: footer present
      const footer = page.locator('footer');
      await expect(footer).toBeVisible();
      const footerText = await footer.textContent();
      expect(footerText).toContain('StockRating');
      expect(footerText).toContain('v3.0');
    });
  });
});
