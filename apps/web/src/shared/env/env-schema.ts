import { z } from "zod";

const ORIGIN_PROTOCOLS = ["http:", "https:"];

const EXPECTED_FORMAT = {
	VITE_API_URL: "expected an absolute URL with no trailing slash (https://…)",
} as const;

type VariableName = keyof typeof EXPECTED_FORMAT;

function isAbsoluteOrigin(value: string): boolean {
	let parsed: URL;
	try {
		parsed = new URL(value);
	} catch {
		return false;
	}
	if (!ORIGIN_PROTOCOLS.includes(parsed.protocol)) {
		return false;
	}
	return parsed.origin === value;
}

const EnvSchema = z.object({
	VITE_API_URL: z.string().refine(isAbsoluteOrigin),
});

export type Env = z.infer<typeof EnvSchema>;

function describeRejection(error: z.ZodError): string {
	const rejected = new Set(error.issues.map((issue) => String(issue.path[0])));
	const names = Object.keys(EXPECTED_FORMAT) as VariableName[];
	const lines = names
		.filter((name) => rejected.has(name))
		.sort()
		.map((name) => `  ${name}: ${EXPECTED_FORMAT[name]}`);
	return ["Invalid environment:", ...lines].join("\n");
}

export function parseEnv(source: Record<string, unknown>): Env {
	const result = EnvSchema.safeParse(source);
	if (result.success) {
		return result.data;
	}
	throw new Error(describeRejection(result.error));
}
