import type { SkillProfile } from "../api/bindings";

class AssessmentStore {
  profile = $state<SkillProfile | null>(null);
  loading = $state(false);
  expanded = $state(false);
}

export const assessmentStore = new AssessmentStore();
