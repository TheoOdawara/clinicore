import { Elysia, t } from "elysia";

export const healthController = new Elysia().get(
	"/health",
	() => ({ status: "ok" as const }),
	{
		response: {
			200: t.Object({
				status: t.Literal("ok"),
			}),
		},
		detail: {
			summary: "Liveness probe",
			tags: ["Health"],
		},
	},
);
