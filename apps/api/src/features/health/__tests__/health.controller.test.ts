import { describe, expect, it } from "bun:test";
import { healthController } from "../controller/health.controller";

describe("GET /health", () => {
	it("answers 200 with the ok status and nothing else", async () => {
		const response = await healthController.handle(
			new Request("http://localhost/health"),
		);

		expect(response.status).toBe(200);
		expect(await response.text()).toBe('{"status":"ok"}');
	});
});
