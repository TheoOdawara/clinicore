import { afterAll, describe, expect, it } from "bun:test";
import { prisma } from "../prisma";

describe("the Prisma client against the development Postgres", () => {
	afterAll(async () => {
		await prisma.$disconnect();
	});

	it("reaches the database without throwing", async () => {
		await expect(prisma.$transaction(async () => {})).resolves.toBeUndefined();
	});
});
