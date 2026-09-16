import { FormatRegistry, type Static, Type } from "@sinclair/typebox";
import { Value } from "@sinclair/typebox/value";

const DIGITS_ONLY = /^[0-9]+$/;
const POSTGRES_PROTOCOLS = ["postgresql:", "postgres:"];
const WEB_ORIGIN_PROTOCOLS = ["http:", "https:"];

function isPostgresUrl(value: string): boolean {
	const parsed = URL.parse(value);
	if (parsed === null) {
		return false;
	}
	return POSTGRES_PROTOCOLS.includes(parsed.protocol) && parsed.hostname !== "";
}

function isWebOrigin(value: string): boolean {
	const parsed = URL.parse(value);
	if (parsed === null) {
		return false;
	}
	if (!WEB_ORIGIN_PROTOCOLS.includes(parsed.protocol)) {
		return false;
	}
	return parsed.origin === value;
}

FormatRegistry.Set("postgres-url", isPostgresUrl);
FormatRegistry.Set("web-origin", isWebOrigin);

const EXPECTED_FORMAT = {
	PORT: "expected an integer between 1 and 65535",
	WEB_ORIGIN: "expected an absolute URL with no trailing slash (https://…)",
	DATABASE_URL: "expected a PostgreSQL connection string (postgresql://…)",
} as const;

type VariableName = keyof typeof EXPECTED_FORMAT;

const EnvSchema = Type.Object({
	DATABASE_URL: Type.String({
		format: "postgres-url",
		description: EXPECTED_FORMAT.DATABASE_URL,
	}),
	PORT: Type.Integer({
		minimum: 1,
		maximum: 65535,
		description: EXPECTED_FORMAT.PORT,
	}),
	WEB_ORIGIN: Type.String({
		format: "web-origin",
		description: EXPECTED_FORMAT.WEB_ORIGIN,
	}),
});

export type Env = Static<typeof EnvSchema>;

function toPort(raw: string | undefined): string | number | undefined {
	if (raw === undefined) {
		return undefined;
	}
	if (!DIGITS_ONLY.test(raw)) {
		return raw;
	}
	return Number(raw);
}

function describeRejection(candidate: unknown): string {
	const rejected = new Set<string>();
	for (const error of Value.Errors(EnvSchema, candidate)) {
		rejected.add(error.path.slice(1));
	}
	const names = Object.keys(EXPECTED_FORMAT) as VariableName[];
	const lines = names
		.filter((name) => rejected.has(name))
		.sort()
		.map((name) => `  ${name}: ${EXPECTED_FORMAT[name]}`);
	return ["Invalid environment:", ...lines].join("\n");
}

export function parseEnv(source: Record<string, string | undefined>): Env {
	const candidate = {
		DATABASE_URL: source.DATABASE_URL,
		PORT: toPort(source.PORT),
		WEB_ORIGIN: source.WEB_ORIGIN,
	};
	if (Value.Check(EnvSchema, candidate)) {
		return candidate;
	}
	throw new Error(describeRejection(candidate));
}
