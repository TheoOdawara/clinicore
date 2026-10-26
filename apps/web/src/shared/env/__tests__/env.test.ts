import { expect, it } from "vitest";
import { env } from "../env";

it("reads VITE_API_URL from the Vite environment", () => {
  expect(env).toEqual({ VITE_API_URL: "http://localhost:3333" });
});
