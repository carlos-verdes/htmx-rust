import { expect, test, type Page } from '@playwright/test';

// A catalog's view holds nested views with addresses of their own: its tabs,
// the tags and the items, and inside the items the one item the address names.
// Each address shows its view inside every view around it, however the reader
// got there. Each scenario runs with and without JavaScript.

function catalog(page: Page, name: string) {
  const outlet = page.locator('main');
  return {
    outlet,
    async expectShownOnceWithItsTabs() {
      await expect(outlet).toHaveCount(1);
      await expect(outlet.getByRole('heading', { level: 1 })).toHaveText(name);
      await expect(outlet.getByRole('navigation').getByRole('link')).toHaveText(['Items', 'Tags']);
    },
  };
}

async function expectOnlyOpenTab(page: Page, open: string) {
  const marked = page.locator('main nav [aria-current]');
  await expect(marked).toHaveText([open]);
}

async function expectSpringTeasTags(page: Page) {
  const { outlet, expectShownOnceWithItsTabs } = catalog(page, 'Spring teas');
  await expect(page).toHaveURL('/catalogs/1/tags');
  await expect(page).toHaveTitle('Tags');
  await expectShownOnceWithItsTabs();
  await expect(outlet.getByRole('heading', { level: 2 })).toHaveText('Tags');
  await expect(outlet.locator('section').getByRole('listitem')).toHaveText(['tea', 'hot']);
  await expectOnlyOpenTab(page, 'Tags');
}

async function expectSummerTeasItems(page: Page, path: string, title: string, item?: string) {
  const { outlet, expectShownOnceWithItsTabs } = catalog(page, 'Summer teas');
  const items = outlet.locator('section');
  await expect(page).toHaveURL(path);
  await expect(page).toHaveTitle(title);
  await expectShownOnceWithItsTabs();
  await expect(items.getByRole('heading', { level: 2 })).toHaveText('Items');
  await expect(items.getByRole('listitem')).toHaveText(['Iced oolong', 'Cold brew green']);
  // The item is below the list, in the outlet of the items.
  const shown = items.locator('ul + div').getByRole('heading', { level: 3 });
  if (item) await expect(shown).toHaveText(item);
  else await expect(shown).toHaveCount(0);
  await expectOnlyOpenTab(page, 'Items');
}

test('a nested view opened at its address, and refreshed, is shown inside its parent', async ({
  page,
}) => {
  await page.goto('/catalogs/1/tags');
  await expectSpringTeasTags(page);

  await page.reload();
  await expectSpringTeasTags(page);
});

test('a view three views deep opened at its address, and refreshed, is shown inside every view around it', async ({
  page,
}) => {
  await page.goto('/catalogs/2/items/Iced%20oolong');
  await expectSummerTeasItems(page, '/catalogs/2/items/Iced%20oolong', 'Item', 'Iced oolong');

  await page.reload();
  await expectSummerTeasItems(page, '/catalogs/2/items/Iced%20oolong', 'Item', 'Iced oolong');
});

test('links between nested views change the address and what is shown inside the catalog', async ({
  page,
  javaScriptEnabled,
}) => {
  // A catalog's own address opens its first tab, at that tab's address.
  await page.goto('/catalogs/2');
  await expectSummerTeasItems(page, '/catalogs/2/items', 'Items');
  const loads: string[] = [];
  page.on('request', (request) => {
    if (request.resourceType() === 'document') loads.push(request.url());
  });

  await page.getByRole('link', { name: 'Cold brew green' }).click();
  await expectSummerTeasItems(page, '/catalogs/2/items/Cold%20brew%20green', 'Item', 'Cold brew green');

  // Another item takes the place of the first.
  await page.getByRole('link', { name: 'Iced oolong' }).click();
  await expectSummerTeasItems(page, '/catalogs/2/items/Iced%20oolong', 'Item', 'Iced oolong');

  // htmx fetches and swaps; without JavaScript each click loads a document.
  expect(loads).toHaveLength(javaScriptEnabled ? 0 : 2);

  await page.goBack();
  await expectSummerTeasItems(page, '/catalogs/2/items/Cold%20brew%20green', 'Item', 'Cold brew green');

  // The item links to the view around it: the same items, with none shown.
  await page.getByRole('link', { name: 'Close' }).click();
  await expectSummerTeasItems(page, '/catalogs/2/items', 'Items');

  // The tags take the place of the items, in the same catalog.
  await page.getByRole('link', { name: 'Tags' }).click();
  await expect(page).toHaveURL('/catalogs/2/tags');
  await catalog(page, 'Summer teas').expectShownOnceWithItsTabs();
  await expect(page.getByRole('heading', { level: 2 })).toHaveText('Tags');
  await expect(page.getByRole('heading', { level: 3 })).toHaveCount(0);
  await expectOnlyOpenTab(page, 'Tags');

  // And back: each tab's answer brings the tabs with it, the open one marked.
  await page.getByRole('link', { name: 'Items' }).click();
  await expectSummerTeasItems(page, '/catalogs/2/items', 'Items');
});
