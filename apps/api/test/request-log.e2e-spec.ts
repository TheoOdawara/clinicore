import request from "supertest";
import {
  createProbeApp,
  PROBE_COOKIE,
  PROBE_SET_COOKIE,
  type Probe,
} from "./probe-app";

const ORIGIN = "https://app.clinicore.com.br";
const PASSWORD = "sup3rs3cret";
const CREDENTIALS = { email: "person@clinicore.com.br", password: PASSWORD };

describe("request log", () => {
  let probe: Probe;

  beforeAll(async () => {
    probe = await createProbeApp();
  });

  afterAll(async () => {
    await probe.app.close();
  });

  it("logs the method, the path, the status and the duration", async () => {
    await request(probe.server)
      .post("/probe/sign-in")
      .set("Origin", ORIGIN)
      .set("Cookie", PROBE_COOKIE)
      .send(CREDENTIALS)
      .expect(201);

    const [line] = probe.linesFor("/probe/sign-in");

    expect(line).toMatchObject({
      method: "POST",
      path: "/probe/sign-in",
      statusCode: 201,
      msg: "request",
    });
    expect(typeof line?.durationMs).toBe("number");
  });

  it("never writes the password, the cookie or the set-cookie", async () => {
    await request(probe.server)
      .post("/probe/sign-in")
      .set("Origin", ORIGIN)
      .set("Cookie", PROBE_COOKIE)
      .send(CREDENTIALS)
      .expect(201);

    expect(probe.written()).not.toContain(PASSWORD);
    expect(probe.written()).not.toContain(PROBE_COOKIE);
    expect(probe.written()).not.toContain(PROBE_SET_COOKIE);
  });

  it("logs the request the guard rejected", async () => {
    await request(probe.server)
      .post("/probe/sign-in")
      .send(CREDENTIALS)
      .expect(403);

    const rejected = probe
      .linesFor("/probe/sign-in")
      .filter((line) => line.statusCode === 403);

    expect(rejected).toMatchObject([{ method: "POST", statusCode: 403 }]);
  });

  it("writes nothing for GET /health", async () => {
    await request(probe.server).get("/health").expect(200);

    expect(probe.linesFor("/health")).toEqual([]);
  });

  it("still logs a POST to the health path", async () => {
    await request(probe.server).post("/health").expect(404);

    expect(probe.linesFor("/health")).toMatchObject([
      { method: "POST", statusCode: 404 },
    ]);
  });
});
