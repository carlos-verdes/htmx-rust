import { expect, test, type Page } from '@playwright/test';

// The URL's state is the view's fields. A GET form is a link whose query string
// comes from its fields, so a search is an address like any other: it can be
// refreshed, gone back to and shared. Each scenario runs with and without
// JavaScript.

/** The names of the catalogs listed. */
function listed(page: Page) {
  return page.locator('#outlet-0').getByRole('listitem').getByRole('link');
}

/** The catalogs listed are exactly these, and the form shows what was searched. */
async function expectCatalogs(page: Page, path: string, searched: string, names: string[]) {
  await expect(page).toHaveURL(path);
  await expect(listed(page)).toHaveText(names);
  await expect(page.getByRole('searchbox', { name: 'Name' })).toHaveValue(searched);
  await expect(page.locator('#outlet-0')).toHaveCount(1);
}

// The catalogs the demo starts with. Other tests start and remove their own
// beside them, so the list without a search is only checked to begin with these.
const first = /^Spring teas$/;
const teas = ['Spring teas', 'Summer teas'];

test('searching updates the address bar, and a refresh shows the same filtered results', async ({
  page,
  javaScriptEnabled,
}) => {
  await page.goto('/catalogs');
  await expect(listed(page).first()).toHaveText(first);
  const loads: string[] = [];
  page.on('request', (request) => {
    if (request.resourceType() === 'document') loads.push(request.url());
  });

  await page.getByRole('searchbox', { name: 'Name' }).fill('tea');
  await page.getByRole('button', { name: 'Search' }).click();

  await expectCatalogs(page, '/catalogs?q=tea', 'tea', teas);
  // htmx sends the form itself; without JavaScript the browser loads its answer.
  expect(loads).toHaveLength(javaScriptEnabled ? 0 : 1);

  await page.reload();
  await expectCatalogs(page, '/catalogs?q=tea', 'tea', teas);

  // Back shows every catalog again. The form is not looked at: without
  // JavaScript the browser itself puts back what was typed in it.
  await page.goBack();
  await expect(page).toHaveURL('/catalogs');
  await expect(listed(page).first()).toHaveText(first);
  await expect(listed(page).nth(3)).toHaveText('Winter coffees');
});

test('a search for what a URL cannot hold as it is comes back the same', async ({ page }) => {
  await page.goto('/catalogs');

  await page.getByRole('searchbox', { name: 'Name' }).fill('r t&=?');
  await page.getByRole('button', { name: 'Search' }).click();

  await expectCatalogs(page, '/catalogs?q=r+t%26%3D%3F', 'r t&=?', []);
  await page.reload();
  await expectCatalogs(page, '/catalogs?q=r+t%26%3D%3F', 'r t&=?', []);
});

test('a result links to its resource, at an address that can be refreshed', async ({ page }) => {
  await page.goto('/catalogs?q=coffee');
  await expectCatalogs(page, '/catalogs?q=coffee', 'coffee', ['Autumn coffees', 'Winter coffees']);

  await page.getByRole('link', { name: 'Winter coffees' }).click();

  const heading = page.locator('#outlet-0').getByRole('heading', { level: 1 });
  await expect(page).toHaveURL('/catalogs/4');
  await expect(heading).toHaveText('Winter coffees');
  await page.reload();
  await expect(page).toHaveURL('/catalogs/4');
  await expect(heading).toHaveText('Winter coffees');

  await page.goBack();
  await expectCatalogs(page, '/catalogs?q=coffee', 'coffee', ['Autumn coffees', 'Winter coffees']);
});
