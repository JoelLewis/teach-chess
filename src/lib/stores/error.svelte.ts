type Severity = "error" | "warning" | "info";

type ErrorOptions = {
  duration?: number;
  severity?: Severity;
  retry?: (() => void) | null;
};

class ErrorStore {
  message = $state<string | null>(null);
  severity = $state<Severity>("error");
  retry = $state<(() => void) | null>(null);
  private timeout: ReturnType<typeof setTimeout> | null = null;

  show(msg: string, optionsOrDuration: ErrorOptions | number = {}) {
    const options: ErrorOptions =
      typeof optionsOrDuration === "number"
        ? { duration: optionsOrDuration }
        : optionsOrDuration;
    const { duration = 5000, severity = "error", retry = null } = options;
    this.message = msg;
    this.severity = severity;
    this.retry = retry;
    if (this.timeout) clearTimeout(this.timeout);
    this.timeout = setTimeout(() => {
      this.message = null;
      this.timeout = null;
      this.retry = null;
    }, duration);
  }

  dismiss() {
    this.message = null;
    this.retry = null;
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = null;
    }
  }
}

export const errorStore = new ErrorStore();
export type { Severity };
