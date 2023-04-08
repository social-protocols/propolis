import { test, expect } from "@playwright/test";

test("create statement", async ({ page }) => {
  await page.goto("http://localhost:8000");

  // create statement
  await page.getByTestId("nav-add-statement").click();
  await page.getByTestId("create-statement-field").click();
  await page.getByTestId("create-statement-field").fill("The earth is flat.");
  await page.getByTestId("create-statement-submit").click();

  // view created statement
  await expect(
    page.getByTestId("current-statement").getByTestId("statement-text")
  ).toHaveText("The earth is flat.");

  // check if it appears in the subscriptions
  await page.getByTestId("nav-my-subscriptions").click();
  await expect(
    page.getByTestId("subscription-statement-0").getByTestId("statement-text")
  ).toHaveText("The earth is flat.");
});
