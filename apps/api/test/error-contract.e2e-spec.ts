import request from "supertest";
import { createProbeApp, type Probe } from "./probe-app";

const ORIGIN = "https://app.clinicore.com.br";

describe("error contract", () => {
  let probe: Probe;

  beforeAll(async () => {
    probe = await createProbeApp();
  });

  afterAll(async () => {
    await probe.app.close();
  });

  it("answers an unhandled error with 500 and no detail", async () => {
    const response = await request(probe.server).get("/probe/boom").expect(500);

    expect(response.body).toEqual({
      code: "INTERNAL_ERROR",
      message: "Internal server error",
      fields: {},
    });
    expect(response.text).not.toContain("probe failure with a stack");
  });

  it("writes the whole stack of the unhandled error to the log", async () => {
    await request(probe.server).get("/probe/boom").expect(500);

    expect(probe.written()).toContain("probe failure with a stack");
    expect(probe.written()).toContain("probe-app.ts");
  });

  it("hides the text of a 500 HttpException and logs it", async () => {
    const response = await request(probe.server)
      .get("/probe/internal")
      .expect(500);

    expect(response.body).toEqual({
      code: "INTERNAL_ERROR",
      message: "Internal server error",
      fields: {},
    });
    expect(response.text).not.toContain("db-prod");
    expect(probe.written()).toContain("db-prod:5432 refused");
  });

  it("keeps the status of a 503 HttpException and hides its text", async () => {
    const response = await request(probe.server)
      .get("/probe/unavailable")
      .expect(503);

    expect(response.body).toEqual({
      code: "SERVICE_UNAVAILABLE",
      message: "Server error",
      fields: {},
    });
    expect(response.text).not.toContain("cache-prod");
    expect(probe.written()).toContain("cache-prod:6379 is down");
  });

  it("answers a business unavailable error with the catalog message", async () => {
    const response = await request(probe.server)
      .get("/probe/business-unavailable")
      .expect(503);

    expect(response.body).toEqual({
      code: "SERVICE_UNAVAILABLE",
      message: "Service temporarily unavailable",
      fields: {},
    });
  });

  it("keeps the status of an exception the framework raised", async () => {
    const response = await request(probe.server)
      .get("/probe/does-not-exist")
      .expect(404);

    expect(response.body).toMatchObject({ code: "NOT_FOUND", fields: {} });
  });

  it("names the first violated constraint of each field", async () => {
    const response = await request(probe.server)
      .post("/probe/sign-in")
      .set("Origin", ORIGIN)
      .send({ email: "not-an-email", password: "short" })
      .expect(400);

    expect(response.body).toEqual({
      code: "VALIDATION_FAILED",
      message: "Validation failed",
      fields: { email: "IS_EMAIL", password: "MIN_LENGTH" },
    });
  });

  it("names the violated constraint of a nested field by its path", async () => {
    const response = await request(probe.server)
      .post("/probe/profile")
      .set("Origin", ORIGIN)
      .send({ address: { zip: 1 } })
      .expect(400);

    expect(response.body).toEqual({
      code: "VALIDATION_FAILED",
      message: "Validation failed",
      fields: { "address.zip": "IS_STRING" },
    });
  });

  it("refuses a field the DTO does not declare", async () => {
    const response = await request(probe.server)
      .post("/probe/sign-in")
      .set("Origin", ORIGIN)
      .send({
        email: "person@clinicore.com.br",
        password: "sup3rs3cret",
        role: "admin",
      })
      .expect(400);

    expect(response.body).toMatchObject({
      code: "VALIDATION_FAILED",
      fields: { role: "WHITELIST_VALIDATION" },
    });
  });
});
