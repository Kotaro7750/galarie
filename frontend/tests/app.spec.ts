import { expect, test } from '@playwright/test'

test.describe('Landing experience', () => {
  test('renders environment overview', async ({ page }) => {
    await page.goto('/')
    await expect(page.getByRole('heading', { name: 'Frontend scaffold ready.' })).toBeVisible()
    await expect(page.getByText('Next setup steps')).toBeVisible()
    await expect(page.getByText('Quickstart', { exact: true })).toBeVisible()
  })
})
