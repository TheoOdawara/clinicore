import request from "supertest";
import { createProbeApp, type Probe } from "./probe-app";

const ORIGIN = "https://app.clinicore.com.br";

describe("auth routes on the whole application", () => {
  let probe: Probe;

  beforeAll(async () => {
    probe = await createProbeApp();
  });

  afterAll(async () => {
    await probe.app.close();
  });

  it("refuses a sign-in without the Origin header before reaching the service", async () => {
    const response = await request(probe.server)
      .post("/sessions")
      .send({ email: "ana@exemplo.com", password: "Clinica#2026" })
      .expect(403);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-origin",
    });
  });

  it("refuses a refresh from an origin outside the list", async () => {
    const response = await request(probe.server)
      .post("/sessions/current/tokens")
      .set("Origin", "http://evil.example")
      .expect(403);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-origin",
    });
  });

  it("refuses a sign-out without the Origin header", async () => {
    const response = await request(probe.server)
      .delete("/sessions/current")
      .expect(403);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-origin",
    });
  });

  it("refuses the session route without the access cookie", async () => {
    const response = await request(probe.server)
      .get("/sessions/current")
      .expect(401);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });
  });

  it("keeps the health check public with the global guard in place", async () => {
    await request(probe.server).get("/health").expect(200);
  });

  it("answers the preflight of an allowed origin with credentials", async () => {
    const response = await request(probe.server)
      .options("/sessions")
      .set("Origin", ORIGIN)
      .set("Access-Control-Request-Method", "POST")
      .expect(204);

    expect(response.headers["access-control-allow-origin"]).toBe(ORIGIN);
    expect(response.headers["access-control-allow-credentials"]).toBe("true");
  });
});
