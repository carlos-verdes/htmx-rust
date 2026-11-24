import { expect, test, type Page } from '@playwright/test';

// Each scenario runs with and without JavaScript. The reader sees the same
// thing; only how it got there differs: htmx fetches and swaps the root view's
// outlet, a browser without JavaScript loads the whole document.

/** Counts the documents the browser loads from now on: none while htmx swaps. */
function documentLoads(page: Page) {
  const loads: string[] = [];
  page.on('request', (request) => {
    if (request.resourceType() === 'document') loads.push(request.url());
  });
  return loads;
}

/** The view the address names is shown, once, in the root view's outlet. */
async function expectView(page: Page, path: string, title: string) {
  await expect(page).toHaveURL(path);
  await expect(page).toHaveTitle(title);
  await expect(page.locator('#outlet-0').getByRole('heading', { level: 1 })).toHaveText(title);
  await expect(page.getByRole('navigation')).toHaveCount(1);
  await expect(page.locator('#outlet-0')).toHaveCount(1);
}

test('a click on a plain link swaps the outlet and updates the address bar', async ({
  page,
  javaScriptEnabled,
}) => {
  await page.goto('/');
  await expectView(page, '/', 'Home');
  const loads = documentLoads(page);

  // A link in the nested view, with no htmx attributes of its own.
  await page.getByRole('link', { name: 'See the catalogs' }).click();
  await expectView(page, '/catalogs', 'Catalogs');

  // A link in the root view's navigation.
  await page.getByRole('navigation').getByRole('link', { name: 'Home' }).click();
  await expectView(page, '/', 'Home');

  expect(loads).toHaveLength(javaScriptEnabled ? 0 : 2);
});

test('refresh, back and forward each show the view the address names', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('link', { name: 'See the catalogs' }).click();
  await expectView(page, '/catalogs', 'Catalogs');

  await page.reload();
  await expectView(page, '/catalogs', 'Catalogs');

  await page.goBack();
  await expectView(page, '/', 'Home');

  await page.goForward();
  await expectView(page, '/catalogs', 'Catalogs');

  await page.goBack();
  await page.reload();
  await expectView(page, '/', 'Home');
});

test('a posted form answered with 303 ends on the view it redirects to', async ({
  page,
  javaScriptEnabled,
}) => {
  await page.goto('/');
  const loads = documentLoads(page);
  const answer = page.waitForResponse((response) => response.request().method() === 'POST');

  await page.getByRole('button', { name: 'Clear the catalogs' }).click();

  expect((await answer).status()).toBe(303);
  await expectView(page, '/catalogs', 'Catalogs');
  // Without JavaScript the browser loads the form's answer and then follows it.
  expect(loads).toHaveLength(javaScriptEnabled ? 0 : 2);

  // The address in the bar is one that can be gone back from and refreshed.
  await page.goBack();
  await expectView(page, '/', 'Home');
  await page.goForward();
  await page.reload();
  await expectView(page, '/catalogs', 'Catalogs');
});

test('a posted form answered with 201 shows the answer in the outlet', async ({
  page,
  javaScriptEnabled,
}) => {
  await page.goto('/');
  const loads = documentLoads(page);
  const answer = page.waitForResponse((response) => response.request().method() === 'POST');

  await page.getByRole('button', { name: 'Start a catalog' }).click();

  expect((await answer).status()).toBe(201);
  // The answer is shown at the address the form posted to, the collection's.
  await expectView(page, '/catalogs', 'Catalog created');
  expect(loads).toHaveLength(javaScriptEnabled ? 0 : 1);

  await page.getByRole('link', { name: 'See the catalogs' }).click();
  await expectView(page, '/catalogs', 'Catalogs');
});
