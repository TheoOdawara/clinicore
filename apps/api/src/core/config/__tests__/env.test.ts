import { describe, expect, it } from "bun:test";
import { parseEnv } from "../env-schema";

const VALID = {
	DATABASE_URL: "postgresql://clinicore:local@localhost:5432/clinicore",
	PORT: "3333",
	WEB_ORIGIN: "http://localhost:3000",
};

function without(...names: string[]): Record<string, string> {
	return Object.fromEntries(
		Object.entries(VALID).filter(([name]) => !names.includes(name)),
	);
}

function rejectionOf(source: Record<string, string | undefined>): string {
	try {
		parseEnv(source);
	} catch (error) {
		if (error instanceof Error) {
			return error.message;
		}
		throw error;
	}
	throw new Error("parseEnv accepted an environment it should have rejected");
}

describe("parseEnv", () => {
	it.each([
		"http://localhost:3000",
		"https://app.clinicore.com.br",
		"http://127.0.0.1:5173",
	])("accepts WEB_ORIGIN %s", (origin) => {
		expect(parseEnv({ ...VALID, WEB_ORIGIN: origin }).WEB_ORIGIN).toBe(origin);
	});

	it("returns PORT as a number, not as the string it came in as", () => {
		const env = parseEnv(VALID);

		expect(env.PORT).toBe(3333);
		expect(env.DATABASE_URL).toBe(VALID.DATABASE_URL);
		expect(env.WEB_ORIGIN).toBe(VALID.WEB_ORIGIN);
	});

	it("rejects a missing DATABASE_URL naming only the expected format", () => {
		expect(rejectionOf(without("DATABASE_URL")).split("\n")).toEqual([
			"Invalid environment:",
			"  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)",
		]);
	});

	it("lists every failing variable in alphabetical order", () => {
		expect(
			rejectionOf(without("WEB_ORIGIN", "DATABASE_URL")).split("\n"),
		).toEqual([
			"Invalid environment:",
			"  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)",
			"  WEB_ORIGIN: expected an absolute URL with no trailing slash (https://…)",
		]);
	});

	it("never prints the value it received", () => {
		const secret = "b4dc0ffee-do-not-leak";

		const message = rejectionOf({
			...VALID,
			DATABASE_URL: `mysql://root:${secret}@localhost:3306/db`,
		});

		expect(message).not.toContain(secret);
		expect(message).toContain(
			"  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)",
		);
	});

	it.each([
		"postgres://clinicore@localhost:5432/clinicore",
		"postgresql://user:pass@db.internal:5432/clinicore?schema=public",
	])("accepts DATABASE_URL %s", (url) => {
		expect(parseEnv({ ...VALID, DATABASE_URL: url }).DATABASE_URL).toBe(url);
	});

	it.each([
		"mysql://localhost:5432/db",
		"localhost:5432",
		"",
		"postgresql",
		"postgresql://",
		"postgresql:///clinicore",
	])("rejects DATABASE_URL %p", (url) => {
		expect(rejectionOf({ ...VALID, DATABASE_URL: url })).toContain(
			"  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)",
		);
	});

	it.each([
		"http://localhost:3000/",
		"localhost:3000",
		"ftp://localhost:3000",
		"http://localhost:3000/api",
		"http://localhost:3000#",
		"http://localhost:3000/#",
		"http://localhost:3000?",
		"http://user:pass@localhost:3000",
		"HTTP://LOCALHOST:3000",
		"https://example.com:443",
		"",
	])("rejects WEB_ORIGIN %p", (origin) => {
		expect(rejectionOf({ ...VALID, WEB_ORIGIN: origin })).toContain(
			"  WEB_ORIGIN: expected an absolute URL with no trailing slash (https://…)",
		);
	});

	it.each([
		"0",
		"65536",
		"abc",
		"3333.5",
		"0x10",
		"1e3",
		" 3333 ",
		"+3333",
		"-3333",
		"",
	])("rejects PORT %p", (port) => {
		expect(rejectionOf({ ...VALID, PORT: port })).toContain(
			"  PORT: expected an integer between 1 and 65535",
		);
	});
});
