// @ts-check
const { test, expect } = require('@playwright/test');

test.describe('API Tests', () => {
  test('/api/query returns valid JSON for AAPL', async ({ request }) => {
    const response = await request.get('/api/query?ticker=AAPL');
    expect(response.ok()).toBeTruthy();
    expect(response.headers()['content-type']).toContain('application/json');
    const body = await response.json();
    expect(body.ticker).toBe('AAPL');
    expect(body).toHaveProperty('company_name');
    expect(body).toHaveProperty('valuation_ratios');
    expect(body).toHaveProperty('financial_health');
    expect(body).toHaveProperty('growth_metrics');
    expect(body).toHaveProperty('market_sentiment');
  });

  test('/api/query returns 404 for unknown ticker', async ({ request }) => {
    const response = await request.get('/api/query?ticker=NOTEXIST');
    expect(response.status()).toBe(404);
  });

  test('/api/query supports provider parameter', async ({ request }) => {
    const response = await request.get('/api/query?ticker=AAPL&provider=second');
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.ticker).toBe('AAPL');
  });

  test('/api/all-stocks returns aggregated data', async ({ request }) => {
    const response = await request.get('/api/all-stocks');
    expect(response.ok()).toBeTruthy();
    expect(response.headers()['content-type']).toContain('application/json');
    const body = await response.json();
    expect(body).toHaveProperty('tickers');
    expect(body).toHaveProperty('chart_groups');
    expect(Array.isArray(body.tickers)).toBeTruthy();
    expect(Array.isArray(body.chart_groups)).toBeTruthy();
  });

  test('/api/all-tickers returns ticker list', async ({ request }) => {
    const response = await request.get('/api/all-tickers');
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toHaveProperty('tickers');
    expect(body).toHaveProperty('total');
    expect(body.total).toBeGreaterThanOrEqual(8);
  });

  test('/api/chart returns chart data', async ({ request }) => {
    const response = await request.get('/api/chart?metrics=pe,roe&chart_type=bar');
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body).toHaveProperty('chart_groups');
  });

  test('all known tickers return data', async ({ request }) => {
    const tickers = ['AAPL', 'MSFT', 'GOOGL', 'TSLA', 'AMZN'];
    for (const ticker of tickers) {
      const response = await request.get(`/api/query?ticker=${ticker}`);
      expect(response.ok()).toBeTruthy();
      const body = await response.json();
      expect(body.ticker).toBe(ticker);
    }
  });
});
