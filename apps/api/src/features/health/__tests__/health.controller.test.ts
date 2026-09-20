import type { Server } from "node:http";
import { Test } from "@nestjs/testing";
import type { INestApplication } from "@nestjs/common";
import request from "supertest";
import { HealthModule } from "../health.module";

describe("GET /health", () => {
  let app: INestApplication;

  beforeAll(async () => {
    const moduleRef = await Test.createTestingModule({
      imports: [HealthModule],
    }).compile();

    app = moduleRef.createNestApplication();
    await app.init();
  });

  afterAll(async () => {
    await app.close();
  });

  it("answers 200 with the liveness body and no indicator", async () => {
    const response = await request(app.getHttpServer() as Server).get(
      "/health",
    );

    expect(response.status).toBe(200);
    expect(response.body).toEqual({
      status: "ok",
      info: {},
      error: {},
      details: {},
    });
  });
});
