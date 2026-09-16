import { afterAll, beforeAll, describe, expect, it } from "bun:test";

const SERVER_ENTRY = Bun.fileURLToPath(
	new URL("../server.ts", import.meta.url),
);
const WEB_ORIGIN = "http://localhost:3000";
const DATABASE_URL = "postgresql://clinicore:local@localhost:5432/clinicore";

type Server = ReturnType<typeof startServer>;

function startServer(env: Record<string, string>) {
	return Bun.spawn([process.execPath, "--no-env-file", SERVER_ENTRY], {
		env,
		stdout: "pipe",
		stderr: "pipe",
	});
}

async function takeFreePort(): Promise<string> {
	const probe = Bun.serve({ port: 0, fetch: () => new Response("") });
	const port = String(probe.port);
	await probe.stop(true);
	return port;
}

async function waitForServer(server: Server, port: string): Promise<void> {
	for (let attempt = 0; attempt < 50; attempt += 1) {
		if (server.exitCode !== null) {
			throw new Error(
				`the server exited with code ${server.exitCode} instead of listening`,
			);
		}
		try {
			await fetch(`http://localhost:${port}/health`);
			return;
		} catch {
			await Bun.sleep(40);
		}
	}
	throw new Error(`the server never answered on port ${port}`);
}

describe("a server booted with a valid environment", () => {
	let server: Server;
	let port: string;

	beforeAll(async () => {
		port = await takeFreePort();
		server = startServer({ DATABASE_URL, PORT: port, WEB_ORIGIN });
		await waitForServer(server, port);
	});

	afterAll(async () => {
		server.kill();
		await server.exited;
	});

	it("answers the health check on the configured port", async () => {
		const response = await fetch(`http://localhost:${port}/health`);

		expect(server.exitCode).toBeNull();
		expect(response.status).toBe(200);
		expect(await response.text()).toBe('{"status":"ok"}');
	});

	it("publishes the health check response schema in the OpenAPI document", async () => {
		const response = await fetch(`http://localhost:${port}/openapi/json`);

		expect(response.status).toBe(200);
		expect(await response.json()).toMatchObject({
			paths: {
				"/health": {
					get: {
						responses: {
							"200": {
								content: {
									"application/json": {
										schema: { properties: { status: { const: "ok" } } },
									},
								},
							},
						},
					},
				},
			},
		});
	});
});

describe("a server booted without DATABASE_URL", () => {
	it("writes the two prescribed lines, exits 1 and leaves the port closed", async () => {
		const port = await takeFreePort();
		const server = startServer({ PORT: port, WEB_ORIGIN });

		const stderr = await new Response(server.stderr).text();
		const exitCode = await server.exited;

		expect(stderr.trimEnd().split("\n")).toEqual([
			"Invalid environment:",
			"  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)",
		]);
		expect(exitCode).toBe(1);
		await expect(fetch(`http://localhost:${port}/health`)).rejects.toThrow();
	});
});
