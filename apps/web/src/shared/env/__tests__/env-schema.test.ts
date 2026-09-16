import { describe, expect, it } from "@jest/globals";
import { parseEnv } from "../env-schema";

const EXPECTED_REJECTION = [
	"Invalid environment:",
	"  VITE_API_URL: expected an absolute URL with no trailing slash (https://…)",
];

function rejectionOf(source: Record<string, unknown>): string {
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
		"http://localhost:3333",
		"https://api.clinicore.com.br",
		"http://127.0.0.1:3333",
	])("accepts VITE_API_URL %s", (url) => {
		expect(parseEnv({ VITE_API_URL: url }).VITE_API_URL).toBe(url);
	});

	it.each([
		"localhost:3333",
		"http://localhost:3333/",
		"https://api.clinicore.com.br/",
		"/api",
		"",
	])("rejects VITE_API_URL %s", (url) => {
		expect(rejectionOf({ VITE_API_URL: url }).split("\n")).toEqual(
			EXPECTED_REJECTION,
		);
	});

	it("rejects a missing VITE_API_URL", () => {
		expect(rejectionOf({}).split("\n")).toEqual(EXPECTED_REJECTION);
	});

	it("never prints the received value", () => {
		const received = "localhost:3333";

		expect(rejectionOf({ VITE_API_URL: received })).not.toContain(received);
	});

	it("ignores variables the schema does not declare", () => {
		const env = parseEnv({
			VITE_API_URL: "http://localhost:3333",
			VITE_UNDECLARED: "kept out",
		});

		expect(env).toEqual({ VITE_API_URL: "http://localhost:3333" });
	});
});
