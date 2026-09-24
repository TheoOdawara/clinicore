import { createHash } from "node:crypto";
import type Redis from "ioredis";
import request from "supertest";
import { freePort } from "../src/__tests__/free-port";
import { REDIS } from "../src/core/redis/redis";
import { PROBE_COOKIE, createProbeApp, type Probe } from "./probe-app";

const ORIGIN = "https://app.clinicore.com.br";
const PROXY_HOP = "10.0.0.1";
const UNKNOWN_ACCOUNT = {
  email: "nobody@clinicore.com.br",
  password: "Wrong#Password1",
};

const SIGN_IN_CLIENT = "203.0.113.25";
const FORWARDED_CLIENT = "203.0.113.7";
const FORGED = [
  "198.51.100.1",
  "198.51.100.2",
  "not-an-ip",
  "127.0.0.1",
  "10.0.0.9",
  "192.0.2.200",
];
const SIGN_UP_CLIENT = "203.0.113.30";
const EMAIL_ROUTE_LIMITS = [
  {
    path: "/email-verifications",
    handler: "requestEmailVerification",
    limit: 3,
    client: "203.0.113.40",
  },
  {
    path: "/password-resets",
    handler: "requestPasswordReset",
    limit: 5,
    client: "203.0.113.41",
  },
  {
    path: "/password-resets/confirmation",
    handler: "confirmPasswordReset",
    limit: 5,
    client: "203.0.113.42",
  },
];
const REFRESH_CLIENT = "203.0.113.31";
const SESSION_READ_CLIENT = "203.0.113.32";
const HEALTH_CLIENT = "203.0.113.33";
const CROSS_ORIGIN_CLIENT = "203.0.113.34";
const LOOPBACK = "127.0.0.1";
const IPV6_NEIGHBOURS = [
  "2001:db8:1:1::1",
  "2001:db8:1:1::2",
  "2001:db8:1:1::3",
  "2001:db8:1:1::4",
  "2001:db8:1:1::5",
];
const IPV6_LATECOMER = "2001:db8:1:1::ffff";
const IPV6_SUBNET = "2001:db8:1:1::/64";

const RATE_LIMITED = {
  type: "tag:clinicore.com.br,2026:rate-limited",
  title: "Too many requests",
  status: 429,
};

const SERVICE_UNAVAILABLE = {
  type: "tag:clinicore.com.br,2026:service-unavailable",
  title: "Service temporarily unavailable",
  status: 503,
};

interface ThrottleKeys {
  hits: string;
  blocked: string;
}

function throttleKeys(
  controller: string,
  handler: string,
  tracker: string,
): ThrottleKeys {
  const hash = createHash("sha256")
    .update(`${controller}-${handler}-default-${tracker}`)
    .digest("hex");
  return {
    hits: `{${hash}:default}:hits`,
    blocked: `{${hash}:default}:blocked`,
  };
}

function signInKeys(tracker: string): ThrottleKeys {
  return throttleKeys("AuthController", "signIn", tracker);
}

function bothOf(keys: ThrottleKeys): string[] {
  return [keys.hits, keys.blocked];
}

const SUITE_KEYS = [
  ...bothOf(signInKeys(SIGN_IN_CLIENT)),
  ...bothOf(signInKeys(FORWARDED_CLIENT)),
  ...FORGED.flatMap((forged) => bothOf(signInKeys(forged))),
  ...bothOf(signInKeys(PROXY_HOP)),
  ...bothOf(signInKeys(CROSS_ORIGIN_CLIENT)),
  ...bothOf(signInKeys(LOOPBACK)),
  ...bothOf(signInKeys(IPV6_SUBNET)),
  ...bothOf(throttleKeys("AuthController", "signUp", SIGN_UP_CLIENT)),
  ...bothOf(throttleKeys("AuthController", "refresh", REFRESH_CLIENT)),
  ...bothOf(throttleKeys("AuthController", "session", SESSION_READ_CLIENT)),
  ...bothOf(throttleKeys("HealthController", "check", HEALTH_CLIENT)),
  ...EMAIL_ROUTE_LIMITS.flatMap((route) =>
    bothOf(throttleKeys("AuthController", route.handler, route.client)),
  ),
];

function forwardedFor(client: string): string {
  return `${client}, ${PROXY_HOP}`;
}

async function statusesOf(
  times: number,
  send: (attempt: number) => request.Test,
): Promise<number[]> {
  const statuses: number[] = [];
  for (let attempt = 0; attempt < times; attempt += 1) {
    const response = await send(attempt);
    statuses.push(response.status);
  }
  return statuses;
}

function repeated(status: number, times: number): number[] {
  return Array.from({ length: times }, () => status);
}

describe("rate limit on the whole application", () => {
  let probe: Probe;
  let redis: Redis;

  function signIn(forwarded: string): request.Test {
    return request(probe.server)
      .post("/sessions")
      .set("Origin", ORIGIN)
      .set("X-Forwarded-For", forwarded)
      .send(UNKNOWN_ACCOUNT);
  }

  beforeAll(async () => {
    probe = await createProbeApp();
    redis = probe.app.get<Redis>(REDIS);
  });

  beforeEach(async () => {
    await redis.del(...SUITE_KEYS);
  });

  afterAll(async () => {
    await redis.del(...SUITE_KEYS);
    await probe.app.close();
  });

  it("answers 401 to five sign-ins from one IP and 429 RATE_LIMITED to the sixth", async () => {
    const statuses = await statusesOf(5, () =>
      signIn(forwardedFor(SIGN_IN_CLIENT)),
    );
    const sixth = await signIn(forwardedFor(SIGN_IN_CLIENT)).expect(429);

    expect(statuses).toEqual(repeated(401, 5));
    expect(sixth.headers["content-type"]).toMatch(
      /^application\/problem\+json/,
    );
    expect(sixth.body).toEqual(RATE_LIMITED);
  });

  it("never limits GET /health, even 200 times in a row", async () => {
    const statuses = await statusesOf(200, () =>
      request(probe.server)
        .get("/health")
        .set("X-Forwarded-For", forwardedFor(HEALTH_CLIENT)),
    );

    expect(statuses.filter((status) => status !== 200)).toEqual([]);
    expect(
      await redis.exists(
        ...bothOf(throttleKeys("HealthController", "check", HEALTH_CLIENT)),
      ),
    ).toBe(0);
  });

  it("counts every forged x-forwarded-for against the client the trusted proxy saw", async () => {
    const statuses = await statusesOf(FORGED.length, (attempt) =>
      signIn(`${FORGED[attempt] ?? ""}, ${FORWARDED_CLIENT}, ${PROXY_HOP}`),
    );

    expect(statuses).toEqual([...repeated(401, 5), 429]);
    expect(await redis.get(signInKeys(FORWARDED_CLIENT).hits)).toBe("6");
    expect(
      await redis.exists(
        ...FORGED.map((forged) => signInKeys(forged).hits),
        signInKeys(PROXY_HOP).hits,
      ),
    ).toBe(0);
  });

  it("limits sign-up to three attempts per minute, invalid bodies included", async () => {
    const signUp = (): request.Test =>
      request(probe.server)
        .post("/users")
        .set("Origin", ORIGIN)
        .set("X-Forwarded-For", forwardedFor(SIGN_UP_CLIENT))
        .send({});

    const statuses = await statusesOf(3, signUp);
    const fourth = await signUp().expect(429);

    expect(statuses).toEqual(repeated(400, 3));
    expect(fourth.body).toEqual(RATE_LIMITED);
  });

  it.each(EMAIL_ROUTE_LIMITS)(
    "limits $path to $limit attempts per minute, invalid bodies included",
    async ({ path, limit, client }) => {
      const send = (): request.Test =>
        request(probe.server)
          .post(path)
          .set("Origin", ORIGIN)
          .set("X-Forwarded-For", forwardedFor(client))
          .send({});

      const statuses = await statusesOf(limit, send);
      const over = await send().expect(429);

      expect(statuses).toEqual(repeated(400, limit));
      expect(over.body).toEqual(RATE_LIMITED);
    },
  );

  it("limits the token refresh to thirty per minute", async () => {
    const refresh = (): request.Test =>
      request(probe.server)
        .post("/sessions/current/tokens")
        .set("Origin", ORIGIN)
        .set("X-Forwarded-For", forwardedFor(REFRESH_CLIENT));

    const statuses = await statusesOf(30, refresh);
    const thirtyFirst = await refresh().expect(429);

    expect(statuses).toEqual(repeated(401, 30));
    expect(thirtyFirst.body).toEqual(RATE_LIMITED);
  });

  it("limits any other route to one hundred per ten seconds, counting before the session check", async () => {
    const readSession = (): request.Test =>
      request(probe.server)
        .get("/sessions/current")
        .set("X-Forwarded-For", forwardedFor(SESSION_READ_CLIENT));

    const statuses = await statusesOf(100, readSession);
    const hundredFirst = await readSession().expect(429);

    expect(statuses).toEqual(repeated(401, 100));
    expect(hundredFirst.body).toEqual(RATE_LIMITED);
  });

  it("refuses a missing Origin before the request is counted", async () => {
    const statuses = await statusesOf(6, () =>
      request(probe.server)
        .post("/sessions")
        .set("Cookie", PROBE_COOKIE)
        .set("X-Forwarded-For", forwardedFor(CROSS_ORIGIN_CLIENT))
        .send(UNKNOWN_ACCOUNT),
    );

    expect(statuses).toEqual(repeated(403, 6));
    expect(await redis.exists(...bothOf(signInKeys(CROSS_ORIGIN_CLIENT)))).toBe(
      0,
    );
  });

  it("tracks a request with no proxy hop as 127.0.0.1", async () => {
    await request(probe.server)
      .post("/sessions")
      .set("Origin", ORIGIN)
      .send(UNKNOWN_ACCOUNT)
      .expect(401);

    expect(await redis.get(signInKeys(LOOPBACK).hits)).toBe("1");
  });

  it("shares one counter across the addresses of an IPv6 /64", async () => {
    const statuses = await statusesOf(IPV6_NEIGHBOURS.length, (attempt) =>
      signIn(forwardedFor(IPV6_NEIGHBOURS[attempt] ?? "")),
    );
    const latecomer = await signIn(forwardedFor(IPV6_LATECOMER)).expect(429);

    expect(statuses).toEqual(repeated(401, 5));
    expect(latecomer.body).toEqual(RATE_LIMITED);
    expect(await redis.get(signInKeys(IPV6_SUBNET).hits)).toBe("6");
  });
});

describe("rate limit with Redis unreachable", () => {
  let probe: Probe;

  beforeAll(async () => {
    const closedPort = await freePort();
    probe = await createProbeApp(`redis://127.0.0.1:${String(closedPort)}`);
  });

  afterAll(async () => {
    await probe.app.close();
  });

  it("fails closed with 503 SERVICE_UNAVAILABLE on a limited route", async () => {
    const response = await request(probe.server)
      .post("/sessions")
      .set("Origin", ORIGIN)
      .set("X-Forwarded-For", forwardedFor(SIGN_IN_CLIENT))
      .send(UNKNOWN_ACCOUNT)
      .expect(503);

    expect(response.body).toEqual(SERVICE_UNAVAILABLE);
  });

  it("keeps GET /health answering 200", async () => {
    await request(probe.server).get("/health").expect(200);
  });
});
