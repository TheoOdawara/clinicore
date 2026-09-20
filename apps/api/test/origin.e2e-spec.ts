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
      code: "INVALID_ORIGIN",
      message: "Invalid origin",
      fields: {},
    });
  });

  it("rejects a POST without the Origin header", async () => {
    const response = await request(probe.server)
      .post("/probe/sign-in")
      .send(CREDENTIALS)
      .expect(403);

    expect(response.body).toMatchObject({ code: "INVALID_ORIGIN" });
  });

  it("answers a GET without the Origin header", async () => {
    await request(probe.server)
      .get("/probe/session")
      .expect(200, { status: "ok" });
  });
});
