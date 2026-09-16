import openapi from "@elysiajs/openapi";
import { Elysia } from "elysia";
import { env } from "./core/config/env";
import { healthController } from "./features/health/controller/health.controller";

new Elysia()
	.use(
		openapi({
			documentation: {
				info: { title: "Clinicore API", version: "0.1.0" },
			},
		}),
	)
	.use(healthController)
	.listen(env.PORT);
