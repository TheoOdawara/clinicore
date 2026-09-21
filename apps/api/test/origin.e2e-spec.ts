import request from "supertest";
import { createProbeApp, type Probe } from "./probe-app";

const CREDENTIALS = {
  email: "person@clinicore.com.br",
  password: "sup3rs3cret",
};

describe("OriginGuard", () => {
  let probe: Probe;

  beforeAll(async () => {
    probe = await createProbeApp();
  });

  afterAll(async () => {
    await probe.app.close();
  });

  it.each(["https://app.clinicore.com.br", "https://clinicore.com.br"])(
    "lets a POST from %p through",
    async (origin) => {
      await request(probe.server)
        .post("/probe/sign-in")
        .set("Origin", origin)
        .send(CREDENTIALS)
        .expect(201);
    },
  );

  it.each([
    "http://evil.example",
    "https://clinicore.com.br.evil.example",
    "https://evil-clinicore.com.br",
    "https://app.clinicore.com.br.evil.example/",
    "https://app.clinicore.com.br.evil.example",
    "https://clinicore.com.br/",
    "HTTPS://CLINICORE.COM.BR",
  ])("rejects a POST from %p", async (origin) => {
    const response = await request(probe.server)
      .post("/probe/sign-in")
      .set("Origin", origin)
      .send(CREDENTIALS)
      .expect(403);

    expect(response.body).toEqual({
      type: "tag:clinicore.com.br,2026:invalid-origin",
      title: "Invalid origin",
      status: 403,
    });
  });

  it("rejects a POST without the Origin header", async () => {
    const response = await request(probe.server)
      .post("/probe/sign-in")
      .send(CREDENTIALS)
      .expect(403);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-origin",
    });
  });

  it.each(["https://app.clinicore.com.br", "https://clinicore.com.br"])(
    "allows the preflight from %p with credentials",
    async (origin) => {
      const response = await request(probe.server)
        .options("/probe/sign-in")
        .set("Origin", origin)
        .set("Access-Control-Request-Method", "POST")
        .expect(204);

      expect(response.headers["access-control-allow-origin"]).toBe(origin);
      expect(response.headers["access-control-allow-credentials"]).toBe("true");
    },
  );

  it("never allows the preflight from an origin outside the list", async () => {
    const response = await request(probe.server)
      .options("/probe/sign-in")
      .set("Origin", "http://evil.example")
      .set("Access-Control-Request-Method", "POST");

    expect(response.headers["access-control-allow-origin"]).toBeUndefined();
  });

  it("answers a GET without the Origin header", async () => {
    await request(probe.server)
      .get("/probe/session")
      .expect(200, { status: "ok" });
  });
});
