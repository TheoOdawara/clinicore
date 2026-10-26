import { z } from "zod";

const expectedFormat = {
  VITE_API_URL: "expected an absolute URL with no trailing slash (https://…)",
} as const;

function isAbsoluteOrigin(value: string): boolean {
  let parsed: URL;
  try {
    parsed = new URL(value);
  } catch {
    return false;
  }
  if (!["http:", "https:"].includes(parsed.protocol)) {
    return false;
  }
  return parsed.origin === value;
}

const envSchema = z.object({
  VITE_API_URL: z.string().refine(isAbsoluteOrigin),
});

export function parseEnv(source: Record<string, unknown>): {
  VITE_API_URL: string;
} {
  const result = envSchema.safeParse(source);
  if (result.success) {
    return result.data;
  }

  const rejected = new Set(
    result.error.issues.map((issue) => String(issue.path[0])),
  );
  const lines = Object.entries(expectedFormat)
    .filter(([name]) => rejected.has(name))
    .map(([name, format]) => `  ${name}: ${format}`);
  throw new Error(["Invalid environment:", ...lines].join("\n"));
}
