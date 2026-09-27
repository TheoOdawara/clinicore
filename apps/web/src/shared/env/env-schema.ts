import { z } from "zod";

export function parseEnv(source: Record<string, unknown>): {
  VITE_API_URL: string;
} {
  const expectedFormat = {
    VITE_API_URL: "expected an absolute URL with no trailing slash (https://…)",
  } as const;
  const isAbsoluteOrigin = (value: string): boolean => {
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
  };
  const envSchema = z.object({
    VITE_API_URL: z.string().refine(isAbsoluteOrigin),
  });

  const result = envSchema.safeParse(source);
  if (result.success) {
    return result.data;
  }

  const rejected = new Set(
    result.error.issues.map((issue) => String(issue.path[0])),
  );
  const names = Object.keys(expectedFormat) as (keyof typeof expectedFormat)[];
  const lines = names
    .filter((name) => rejected.has(name))
    .sort()
    .map((name) => `  ${name}: ${expectedFormat[name]}`);
  throw new Error(["Invalid environment:", ...lines].join("\n"));
}
