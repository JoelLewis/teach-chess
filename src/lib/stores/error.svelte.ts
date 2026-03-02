class ErrorStore {
  message = $state<string | null>(null);
  private timeout: ReturnType<typeof setTimeout> | null = null;

  show(msg: string, durationMs = 5000) {
    this.message = msg;
    if (this.timeout) clearTimeout(this.timeout);
    this.timeout = setTimeout(() => {
      this.message = null;
      this.timeout = null;
    }, durationMs);
  }

  dismiss() {
    this.message = null;
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = null;
    }
  }
}

export const errorStore = new ErrorStore();
