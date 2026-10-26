import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig(({ mode }) => ({
  plugins: [
    tanstackRouter({
      target: "react",
      autoCodeSplitting: true,
      codeSplittingOptions: { addHmr: mode !== "test" },
    }),
    react(),
    tailwindcss(),
  ],
  server: {
    port: 3000,
  },
  test: {
    environment: "happy-dom",
    include: ["src/**/__tests__/**/*.test.{ts,tsx}"],
    env: { VITE_API_URL: "http://localhost:3333" },
    sequence: { shuffle: true },
    coverage: {
      provider: "v8",
      include: ["src/**/*.{ts,tsx}"],
      exclude: ["src/routeTree.gen.ts", "src/**/__tests__/**"],
      thresholds: { 100: true },
    },
  },
}));
