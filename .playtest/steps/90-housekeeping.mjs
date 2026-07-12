// End-of-run checks: no error toast anywhere, screenshot for the record.
import { assertNoErrorToast } from "./_helpers.mjs";

export default async (d) => {
  await assertNoErrorToast(d);
  await d.screenshot("90-final");
};
