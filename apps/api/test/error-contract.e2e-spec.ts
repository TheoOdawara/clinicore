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
