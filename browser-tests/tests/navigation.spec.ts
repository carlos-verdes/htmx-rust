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

/** Starts a catalog from the home view and gives back its name. */
async function startCatalog(page: Page) {
  await page.goto('/');
  await page.getByRole('button', { name: 'Start a catalog' }).click();
  await expectView(page, '/catalogs', 'Catalog created');
  return page.locator('#outlet-0 strong').innerText();
}

test('a posted form answered with 201 shows the answer, and the catalog is listed', async ({
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
  const name = await page.locator('#outlet-0 strong').innerText();

  // The answer links to the catalog it created, at an address of its own.
  await page.getByRole('link', { name: `View ${name}`, exact: true }).click();
  await expect(page).toHaveURL(/\/catalogs\/\d+$/);
  await expect(page.locator('#outlet-0').getByRole('heading', { level: 1 })).toHaveText(name);

  await page.getByRole('link', { name: 'All catalogs' }).click();
  await expectView(page, '/catalogs', 'Catalogs');
  await expect(page.getByRole('button', { name: `Remove ${name}`, exact: true })).toHaveCount(1);
});

test('a posted form answered with 303 ends on the view it redirects to, changed', async ({
  page,
  javaScriptEnabled,
}) => {
  const name = await startCatalog(page);
  const remove = page.getByRole('button', { name: `Remove ${name}`, exact: true });
  await page.goto('/');
  await page.getByRole('link', { name: 'See the catalogs' }).click();
  // The form is sent from a search's address, which is not the one it redirects to.
  await page.getByRole('searchbox', { name: 'Name' }).fill(name);
  await page.getByRole('button', { name: 'Search' }).click();
  await expect(page).toHaveURL(/\/catalogs\?q=/);
  await expect(remove).toHaveCount(1);
  const loads = documentLoads(page);
  const answer = page.waitForResponse((response) => response.request().method() === 'POST');

  await remove.click();

  expect((await answer).status()).toBe(303);
  await expect(remove).toHaveCount(0);
  // The bar shows the view redirected to, not the address the form posted to.
  await expectView(page, '/catalogs', 'Catalogs');
  // Without JavaScript the browser loads the form's answer and then follows it.
  expect(loads).toHaveLength(javaScriptEnabled ? 0 : 2);

  // That address can be refreshed, and the catalog is still gone.
  await page.reload();
  await expectView(page, '/catalogs', 'Catalogs');
  await expect(remove).toHaveCount(0);

  // The form's answer is a step of its own in the history: one step back is
  // the search it was sent from, the next the catalogs, and one more is home.
  await page.goBack();
  await expect(page).toHaveURL(/\/catalogs\?q=/);
  await page.goBack();
  await expectView(page, '/catalogs', 'Catalogs');
  await page.goBack();
  await expectView(page, '/', 'Home');
});
