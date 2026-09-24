import request from "supertest";
import { PROBE_COOKIE, createProbeApp, type Probe } from "./probe-app";

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

  it("rejects a POST without the Origin header that carries a cookie", async () => {
    const response = await request(probe.server)
      .post("/probe/sign-in")
      .set("Cookie", PROBE_COOKIE)
      .send(CREDENTIALS)
      .expect(403);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-origin",
    });
  });

  it.each(["delete", "put", "patch"] as const)(
    "rejects a %s without the Origin header that carries a cookie",
    async (method) => {
      const response = await request(probe.server)
        [method]("/probe/resource")
        .set("Cookie", PROBE_COOKIE)
        .expect(403);

      expect(response.body).toMatchObject({
        type: "tag:clinicore.com.br,2026:invalid-origin",
      });
    },
  );

  it.each(["delete", "put", "patch"] as const)(
    "lets a %s from an allowed origin through",
    async (method) => {
      await request(probe.server)
        [method]("/probe/resource")
        .set("Origin", "https://app.clinicore.com.br")
        .expect(200, { status: "ok" });
    },
  );

  it("lets a POST without Origin and without Cookie through", async () => {
    await request(probe.server)
      .post("/probe/sign-in")
      .send(CREDENTIALS)
      .expect(201);
  });

  it.each(["delete", "put", "patch"] as const)(
    "lets a %s without Origin and without Cookie through",
    async (method) => {
      await request(probe.server)
        [method]("/probe/resource")
        .expect(200, { status: "ok" });
    },
  );

  it("rejects a POST from an origin outside the list even without a cookie", async () => {
    const response = await request(probe.server)
      .post("/probe/sign-in")
      .set("Origin", "https://evil.example")
      .send(CREDENTIALS)
      .expect(403);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-origin",
    });
  });

  it("answers a HEAD without the Origin header", async () => {
    await request(probe.server).head("/probe/resource").expect(200);
  });

  it.each(["DELETE", "PUT", "PATCH"])(
    "allows the %s preflight from an allowed origin",
    async (method) => {
      const response = await request(probe.server)
        .options("/probe/resource")
        .set("Origin", "https://app.clinicore.com.br")
        .set("Access-Control-Request-Method", method)
        .expect(204);

      expect(response.headers["access-control-allow-methods"]).toContain(
        method,
      );
    },
  );

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
