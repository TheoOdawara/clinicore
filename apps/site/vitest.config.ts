import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "happy-dom",
    include: ["src/**/__tests__/**/*.test.{ts,tsx}"],
    passWithNoTests: true,
    sequence: { shuffle: true },
    coverage: {
      provider: "v8",
      include: ["src/**/*.{ts,tsx}"],
      exclude: ["src/app/**", "src/**/__tests__/**"],
      thresholds: { 100: true },
    },
  },
});
