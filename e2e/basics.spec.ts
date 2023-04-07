import { test, expect } from "@playwright/test";

test("add statement", async ({ page }) => {
  await page.goto("http://localhost:8000/statement/6");
  await page.getByRole("link", { name: "Add Statement" }).click();
  await page
    .getByPlaceholder(
      "Careful, this is a new statement to be understood independently. It's not a reply."
    )
    .click();
  await page
    .getByPlaceholder(
      "Careful, this is a new statement to be understood independently. It's not a reply."
    )
    .click();
  await page
    .getByPlaceholder(
      "Careful, this is a new statement to be understood independently. It's not a reply."
    )
    .fill("hello");
  await page
    .getByPlaceholder(
      "Careful, this is a new statement to be understood independently. It's not a reply."
    )
    .press("Control+a");
  await page
    .getByPlaceholder(
      "Careful, this is a new statement to be understood independently. It's not a reply."
    )
    .fill("The earth is flat.");
  await page.getByRole("button", { name: "Add Statement" }).click();
  await page.getByText("The earth is flat.").click();
  await page.getByRole("link", { name: "My Subscriptions" }).click();
  await page
    .locator("div")
    .filter({ hasText: /^The earth is flat\.$/ })
    .click();
});
