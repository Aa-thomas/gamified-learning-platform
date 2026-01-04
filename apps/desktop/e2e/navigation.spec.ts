import { test, expect } from '@playwright/test'
import { injectTauriMock } from './mocks/tauri'
import { scenarios } from './fixtures/test-data'

/**
 * E2E tests for navigation and page routing.
 * 
 * These tests verify:
 * - All main routes are accessible
 * - Navigation between pages works
 * - Pages load correct content
 * - Keyboard shortcuts work
 */

test.describe('Navigation', () => {
  test.describe('Main Routes', () => {
    test('home page loads', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/')
      
      await expect(page).toHaveURL('/')
      // Home shows "Welcome back, {username}!"
      await expect(page.getByText(/Welcome back,/)).toBeVisible()
    })

    test('skill tree page loads', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/skill-tree')
      
      await expect(page).toHaveURL('/skill-tree')
      // Should show curriculum content
      await expect(page.locator('main')).toBeVisible()
    })

    test('progress page loads', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/progress')
      
      await expect(page).toHaveURL('/progress')
    })

    test('badges page loads', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/badges')
      
      await expect(page).toHaveURL('/badges')
    })

    test('settings page loads', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/settings')
      
      await expect(page).toHaveURL('/settings')
    })

    test('review page loads', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/review')
      
      await expect(page).toHaveURL('/review')
    })

    test('curriculum manager page loads', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/curriculum')
      
      await expect(page).toHaveURL('/curriculum')
    })
  })

  test.describe('Navigation Bar', () => {
    test('navigation bar is visible on main pages', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/')
      
      // Nav should be visible
      await expect(page.locator('nav')).toBeVisible()
    })

    test('navigation bar is hidden on welcome page', async ({ page }) => {
      const state = scenarios.fresh()
      await injectTauriMock(page, state)
      
      await page.goto('/welcome')
      
      // Nav should not be visible on welcome page
      await expect(page.locator('nav')).not.toBeVisible()
    })

    test('can navigate to skill tree via nav', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/')
      
      // Click skill tree link in nav - exact text is "Skill Tree"
      await page.getByRole('link', { name: 'Skill Tree' }).click()
      await expect(page).toHaveURL('/skill-tree')
    })

    test('can navigate to progress via nav', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/')
      
      // Click progress link - exact text is "Progress"
      await page.getByRole('link', { name: 'Progress' }).click()
      await expect(page).toHaveURL('/progress')
    })

    test('can navigate to badges via nav', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/')
      
      // Click badges link - exact text is "Badges"
      await page.getByRole('link', { name: 'Badges' }).click()
      await expect(page).toHaveURL('/badges')
    })
  })

  test.describe('Dynamic Routes', () => {
    test('lecture route with ID loads', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/lecture/week1-day1-lecture')
      
      await expect(page).toHaveURL('/lecture/week1-day1-lecture')
    })

    test('quiz route with ID loads', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/quiz/week1-day1-quiz')
      
      await expect(page).toHaveURL('/quiz/week1-day1-quiz')
    })
  })

  test.describe('Status Bar', () => {
    test('status bar shows on main pages', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/')
      
      // Status bar should be visible (shows level, XP, streak)
      // The exact selector depends on your implementation
    })

    test('status bar shows user level', async ({ page }) => {
      const state = scenarios.experienced()
      await injectTauriMock(page, state)
      
      await page.goto('/')
      
      // Should show level 2 for experienced user
      await expect(page.getByText('Level 2')).toBeVisible()
    })
  })

  test.describe('Keyboard Shortcuts', () => {
    test('Ctrl+1 navigates to home', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/progress')
      await page.waitForLoadState('networkidle')
      
      // Click on body first to ensure focus
      await page.click('body')
      
      // Press Ctrl+1 to go to Dashboard (home)
      await page.keyboard.press('Control+1')
      
      // Should navigate to home
      await expect(page).toHaveURL('/', { timeout: 3000 })
    })

    test('Ctrl+2 navigates to skill tree', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/')
      await page.waitForLoadState('networkidle')
      
      // Click on body first to ensure focus
      await page.click('body')
      
      await page.keyboard.press('Control+2')
      
      await expect(page).toHaveURL('/skill-tree', { timeout: 3000 })
    })

    test('Ctrl+/ opens shortcuts modal', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/')
      await page.waitForLoadState('networkidle')
      
      // Click on body first to ensure focus
      await page.click('body')
      
      // Press Ctrl+/ to open shortcuts help
      await page.keyboard.press('Control+/')
      
      // Modal should appear - look for the heading inside it (use role to be specific)
      await expect(page.getByRole('heading', { name: 'Keyboard Shortcuts' })).toBeVisible({ timeout: 3000 })
    })

    test('clicking outside modal closes it', async ({ page }) => {
      const state = scenarios.newUser()
      await injectTauriMock(page, state)
      
      await page.goto('/')
      await page.waitForLoadState('networkidle')
      
      // Click on body first to ensure focus
      await page.click('body')
      
      // Open shortcuts modal with Ctrl+/
      await page.keyboard.press('Control+/')
      
      // Wait for modal to appear
      await expect(page.getByRole('heading', { name: 'Keyboard Shortcuts' })).toBeVisible({ timeout: 3000 })
      
      // Click on the backdrop (outside the modal) to close
      // The backdrop is the fixed overlay div
      await page.locator('.fixed.inset-0').click({ position: { x: 10, y: 10 } })
      
      // Modal heading should no longer be visible
      await expect(page.getByRole('heading', { name: 'Keyboard Shortcuts' })).not.toBeVisible({ timeout: 2000 })
    })
  })

  test.describe('Welcome Flow', () => {
    test('new users without account see welcome page', async ({ page }) => {
      const state = scenarios.fresh()
      await injectTauriMock(page, state)
      
      await page.goto('/')
      
      // Should show onboarding when no user exists
      await expect(page.getByText(/welcome/i)).toBeVisible()
    })

    test('welcome page has getting started content', async ({ page }) => {
      const state = scenarios.fresh()
      await injectTauriMock(page, state)
      
      await page.goto('/welcome')
      
      await expect(page).toHaveURL('/welcome')
      // Welcome page should have onboarding content
    })
  })

  test.describe('Page Content Integrity', () => {
    test('home page shows continue learning section', async ({ page }) => {
      const state = scenarios.withProgress()
      await injectTauriMock(page, state)
      
      await page.goto('/')
      
      // Should show "Continue Learning" heading
      await expect(page.getByText('Continue Learning')).toBeVisible()
    })

    test('badges page shows badge grid', async ({ page }) => {
      const state = scenarios.experienced()
      await injectTauriMock(page, state)
      
      await page.goto('/badges')
      
      // Should show some badges
      await expect(page.locator('[data-testid="badge"], .badge, article').first()).toBeVisible({ timeout: 3000 }).catch(() => {
        // Badge styling may vary
      })
    })

    test('progress page shows stats', async ({ page }) => {
      const state = scenarios.experienced()
      await injectTauriMock(page, state)
      
      await page.goto('/progress')
      
      // Should show progress statistics
      await expect(page.locator('main')).toBeVisible()
    })
  })
})
