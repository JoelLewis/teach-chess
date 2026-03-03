import type { SkillProfile } from "../types/assessment";

class AssessmentStore {
  profile = $state<SkillProfile | null>(null);
  loading = $state(false);
  expanded = $state(false);
}

export const assessmentStore = new AssessmentStore();
