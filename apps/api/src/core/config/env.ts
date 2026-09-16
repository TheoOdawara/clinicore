import { type Env, parseEnv } from "./env-schema";

function loadEnv(): Env {
	try {
		return parseEnv(Bun.env);
	} catch (error) {
		if (!(error instanceof Error)) {
			throw error;
		}
		process.stderr.write(`${error.message}\n`);
		process.exit(1);
	}
}

export const env = loadEnv();
