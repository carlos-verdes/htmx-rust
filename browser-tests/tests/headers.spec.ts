import { expect, test, type Page, type Request } from '@playwright/test';

// What htmx 4 sends is what the server's cut reads (ADR 1). These tests pin it,
// so that an htmx upgrade that changes a header fails here first.

test.skip(({ javaScriptEnabled }) => !javaScriptEnabled, 'only htmx sends these headers');

/** Collects every request the page sends from now on. */
function requestsOf(page: Page) {
  const requests: Request[] = [];
  page.on('request', (request) => requests.push(request));
  return requests;
}

/** Each request as its method and path, and the htmx headers it carried. */
async function htmxHeaders(requests: Request[]) {
  return Promise.all(
    requests.map(async (request) => {
      const headers = Object.entries(await request.allHeaders());
      return {
        request: `${request.method()} ${new URL(request.url()).pathname}`,
        ...Object.fromEntries(headers.filter(([name]) => name.startsWith('hx-'))),
      };
    }),
  );
}

test('a click on a link names the outlet it will fill', async ({ page, baseURL }) => {
  await page.goto('/');
  const requests = requestsOf(page);
  await page.getByRole('link', { name: 'See the catalogs' }).click();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Catalogs');

  expect(await htmxHeaders(requests)).toEqual([
    {
      request: 'GET /catalogs',
      'hx-request': 'true',
      'hx-target': 'main#outlet-0',
      'hx-source': 'a',
      'hx-boosted': 'true',
      'hx-current-url': `${baseURL}/`,
      // "full" because the demo still cuts in the browser, with `hx-select`.
      'hx-request-type': 'full',
    },
  ]);
});

test('a posted form names the outlet, and so does the redirect that follows', async ({
  page,
  baseURL,
}) => {
  await page.goto('/');
  const requests = requestsOf(page);
  await page.getByRole('button', { name: 'Start a catalog' }).click();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Catalogs');

  const headers = {
    'hx-request': 'true',
    'hx-target': 'main#outlet-0',
    'hx-source': 'form',
    'hx-boosted': 'true',
    'hx-current-url': `${baseURL}/`,
    'hx-request-type': 'full',
  };
  expect(await htmxHeaders(requests)).toEqual([
    { request: 'POST /catalogs', ...headers },
    { request: 'GET /catalogs', ...headers },
  ]);
});

test('a history restore names no outlet', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('link', { name: 'See the catalogs' }).click();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Catalogs');
  const requests = requestsOf(page);
  await page.goBack();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Home');

  expect(await htmxHeaders(requests)).toEqual([
    {
      request: 'GET /',
      'hx-history-restore-request': 'true',
      'hx-request-type': 'full',
    },
  ]);
});
