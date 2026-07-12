// Fresh-user onboarding: reset the flag, reload, skip → lands on home.
import { assert } from "./_helpers.mjs";

export default async (d) => {
  await d.js(
    `(() => {
      localStorage.removeItem("chessMentor.onboardingComplete");
      setTimeout(() => location.reload(), 50);
      return true;
    })()`,
  );
  await d.sleep(1500);
  await d.waitFor("!!window.__playtest", { timeoutMs: 15_000 });

  // First-run onboarding offers a starter puzzle with a skip button.
  await d.waitFor('!!document.querySelector("button.skip-link")', { timeoutMs: 10_000 });
  await d.screenshot("00-onboarding");
  await d.js('document.querySelector("button.skip-link").click()');

  await d.waitFor('window.__playtest.getView() === "home"', { timeoutMs: 5_000 });
  const flag = await d.js('localStorage.getItem("chessMentor.onboardingComplete")');
  assert.equal(flag, "true", "onboarding flag not persisted");
  await d.screenshot("00-home-after-skip");
};
