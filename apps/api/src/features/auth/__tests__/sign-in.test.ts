import { Session } from "../entities/session.entity";
import { User } from "../entities/user.entity";
import {
  cookieNamed,
  createAuthApp,
  createVerifiedUser,
  type AuthApp,
} from "./auth-app";

const EMAIL = "ana@exemplo.com";
const PASSWORD = "Clinica#2026";
const ACCESS_COOKIE = "clinicore_access";
const REFRESH_COOKIE = "clinicore_refresh";
const DAY_IN_MILLISECONDS = 24 * 60 * 60 * 1000;
const TOLERANCE_IN_MILLISECONDS = 60 * 1000;
const SAMPLE_SIZE = 20;
const MEDIAN_TOLERANCE_IN_MILLISECONDS = 50;

function median(values: number[]): number {
  const sorted = [...values].sort((first, second) => first - second);
  const middle = sorted[Math.floor(sorted.length / 2)];

  if (middle === undefined) {
    throw new Error("median of an empty sample");
  }

  return middle;
}

describe("POST /sessions", () => {
  let authApp: AuthApp;

  beforeAll(async () => {
    authApp = await createAuthApp();
  });

  beforeEach(async () => {
    await authApp.reset();
  });

  afterAll(async () => {
    await authApp.close();
  });

  it("opens the session with both cookies", async () => {
    const user = await createVerifiedUser(authApp, EMAIL, PASSWORD);

    const response = await authApp
      .post("/sessions")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(201);

    expect(response.body).toEqual({
      user: {
        id: user.id,
        name: "Ana Souza",
        email: EMAIL,
        emailVerified: true,
        image: null,
      },
    });
    expect(response.headers.location).toBe("/sessions/current");
    expect(JSON.stringify(response.body)).not.toContain(PASSWORD);

    const access = cookieNamed(response, ACCESS_COOKIE);
    const refresh = cookieNamed(response, REFRESH_COOKIE);
    expect(access).toContain("HttpOnly");
    expect(access).toContain("Path=/");
    expect(access).toContain("Max-Age=900");
    expect(access).toContain("SameSite=Lax");
    expect(access).not.toContain("Domain");
    expect(refresh).toContain("HttpOnly");
    expect(refresh).toContain("Path=/sessions/current/tokens");
    expect(refresh).toContain("Max-Age=86400");
    expect(refresh).toContain("SameSite=Lax");

    const session = await authApp.dataSource
      .getRepository(Session)
      .findOneOrFail({ where: { userId: user.id } });
    const lifetime = session.expiresAt.getTime() - session.createdAt.getTime();
    expect(Math.abs(lifetime - DAY_IN_MILLISECONDS)).toBeLessThanOrEqual(
      TOLERANCE_IN_MILLISECONDS,
    );
    expect(refresh).not.toContain(session.refreshTokenHash);

    const me = await authApp
      .get("/sessions/current")
      .set("Cookie", [access ?? ""])
      .expect(200);
    expect(me.body).toEqual(response.body);
  });

  it("answers the wrong password and the unknown email the same way", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);

    const wrongPassword = await authApp
      .post("/sessions")
      .send({ email: EMAIL, password: "Errada#2026" })
      .expect(401);
    const unknownEmail = await authApp
      .post("/sessions")
      .send({ email: "ninguem@exemplo.com", password: "Errada#2026" })
      .expect(401);

    expect(wrongPassword.body).toEqual({
      type: "tag:clinicore.com.br,2026:invalid-credentials",
      title: "Invalid email or password",
      status: 401,
    });
    expect(unknownEmail.body).toEqual(wrongPassword.body);
  });

  it("takes the same time for the wrong password and the unknown email", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);

    const measure = async (email: string): Promise<number> => {
      const started = performance.now();
      await authApp
        .post("/sessions")
        .send({ email, password: "Errada#2026" })
        .expect(401);
      return performance.now() - started;
    };

    const wrongPassword: number[] = [];
    const unknownEmail: number[] = [];
    for (let attempt = 0; attempt < SAMPLE_SIZE; attempt += 1) {
      wrongPassword.push(await measure(EMAIL));
      unknownEmail.push(await measure("ninguem@exemplo.com"));
    }

    const difference = Math.abs(median(wrongPassword) - median(unknownEmail));
    expect(difference).toBeLessThan(MEDIAN_TOLERANCE_IN_MILLISECONDS);
  });

  it("refuses the sign-in of an unverified email", async () => {
    await authApp
      .post("/users")
      .send({ name: "Ana Souza", email: EMAIL, password: PASSWORD })
      .expect(202);

    const response = await authApp
      .post("/sessions")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(403);

    expect(response.body).toEqual({
      type: "tag:clinicore.com.br,2026:email-not-verified",
      title: "Email not verified",
      status: 403,
    });
    expect(await authApp.dataSource.getRepository(Session).count()).toBe(0);
  });

  it("finds the user by the email in any case", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);

    await authApp
      .post("/sessions")
      .send({ email: "ANA@Exemplo.com", password: PASSWORD })
      .expect(201);

    expect(await authApp.dataSource.getRepository(User).count()).toBe(1);
  });
});
