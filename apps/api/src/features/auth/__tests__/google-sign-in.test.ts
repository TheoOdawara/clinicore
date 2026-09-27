import nock from "nock";
import type request from "supertest";
import { EnvironmentService } from "../../../core/config/environment.service";
import { Account } from "../entities/account.entity";
import { Session } from "../entities/session.entity";
import { User } from "../entities/user.entity";
import {
  Provider,
  cookieNamed,
  cookiesOf,
  createAuthApp,
  createVerifiedUser,
  valueOf,
  type AuthApp,
} from "./auth-app";

const EMAIL = "ana@exemplo.com";
const PASSWORD = "Clinica#2026";
const GOOGLE_SUBJECT = "google-ana";
const STATE_COOKIE = "clinicore_oauth_state";
const ACCESS_COOKIE = "clinicore_access";
const REFRESH_COOKIE = "clinicore_refresh";
const TOKEN_ORIGIN = "https://oauth2.googleapis.com";
const TOKEN_PATH = "/token";
const ACCOUNT_COLUMNS = [
  "createdAt",
  "id",
  "passwordHash",
  "provider",
  "providerAccountId",
  "updatedAt",
  "userId",
];

interface GoogleUserinfo {
  sub: string;
  email: string;
  email_verified: boolean;
  name: string;
  picture?: string;
}

const ANA_AT_GOOGLE: GoogleUserinfo = {
  sub: GOOGLE_SUBJECT,
  email: EMAIL,
  email_verified: true,
  name: "Ana Google",
};

function stubGoogle(userinfo: GoogleUserinfo): void {
  nock(TOKEN_ORIGIN).post(TOKEN_PATH).reply(200, {
    access_token: "google-access-token",
    refresh_token: "google-refresh-token",
    id_token: "google-id-token",
    token_type: "Bearer",
    expires_in: 3599,
  });
  nock("https://www.googleapis.com")
    .get("/oauth2/v3/userinfo")
    .query({ access_token: "google-access-token" })
    .reply(200, userinfo);
}

async function startSignIn(
  authApp: AuthApp,
): Promise<{ state: string; stateCookie: string }> {
  const response = await authApp.get("/oauth/google").expect(302);
  const state = new URL(response.headers.location ?? "").searchParams.get(
    "state",
  );
  const stateCookie = cookieNamed(response, STATE_COOKIE);

  if (state === null || stateCookie === undefined) {
    throw new Error("GET /oauth/google issued no state");
  }

  return { state, stateCookie: `${STATE_COOKIE}=${valueOf(stateCookie)}` };
}

function callback(
  authApp: AuthApp,
  query: Record<string, string>,
  stateCookie?: string,
): request.Test {
  const search = new URLSearchParams(query).toString();
  const sent = authApp.get(`/oauth/google/callback?${search}`);

  if (stateCookie === undefined) {
    return sent;
  }

  return sent.set("Cookie", stateCookie);
}

async function completeSignIn(authApp: AuthApp): Promise<request.Response> {
  const started = await startSignIn(authApp);

  return callback(
    authApp,
    { code: "google-code", state: started.state },
    started.stateCookie,
  );
}

async function rowCounts(authApp: AuthApp): Promise<Record<string, number>> {
  const counts: Record<string, number> = {};

  for (const metadata of authApp.dataSource.entityMetadatas) {
    counts[metadata.tableName] = await authApp.dataSource
      .getRepository(metadata.target)
      .count();
  }

  return counts;
}

async function authRowCounts(authApp: AuthApp): Promise<number[]> {
  return Promise.all([
    authApp.dataSource.getRepository(User).count(),
    authApp.dataSource.getRepository(Account).count(),
    authApp.dataSource.getRepository(Session).count(),
  ]);
}

describe("sign-in with Google", () => {
  let authApp: AuthApp;

  beforeAll(async () => {
    nock.disableNetConnect();
    nock.enableNetConnect(/127\.0\.0\.1|localhost/);
    authApp = await createAuthApp();
  });

  beforeEach(async () => {
    nock.cleanAll();
    await authApp.reset();
  });

  afterAll(async () => {
    await authApp.close();
    nock.cleanAll();
    nock.enableNetConnect();
  });

  describe("GET /oauth/google", () => {
    it("redirects to Google with the state cookie and writes nothing", async () => {
      const before = await rowCounts(authApp);
      const clientId = authApp.app
        .get(EnvironmentService)
        .get("GOOGLE_CLIENT_ID");
      const apiUrl = authApp.app.get(EnvironmentService).get("API_URL");

      for (let attempt = 0; attempt < 3; attempt += 1) {
        const response = await authApp.get("/oauth/google").expect(302);
        const location = new URL(response.headers.location ?? "");
        const stateCookie = cookieNamed(response, STATE_COOKIE);

        expect(location.host).toBe("accounts.google.com");
        expect(location.searchParams.get("prompt")).toBe("select_account");
        expect(location.searchParams.get("scope")).toBe("openid email profile");
        expect(location.searchParams.get("client_id")).toBe(clientId);
        expect(location.searchParams.get("redirect_uri")).toBe(
          `${apiUrl}/oauth/google/callback`,
        );
        expect(stateCookie).toContain("HttpOnly");
        expect(stateCookie).toContain("Path=/oauth/google");
        expect(stateCookie).toContain("Max-Age=600");
        expect(stateCookie).toContain("SameSite=Lax");
        expect(valueOf(stateCookie ?? "")).toMatch(/^[A-Za-z0-9_-]{43}$/);
        expect(location.searchParams.get("state")).toBe(
          valueOf(stateCookie ?? ""),
        );
      }

      expect(await rowCounts(authApp)).toEqual(before);
    });

    it("issues a different state on every request", async () => {
      const first = await startSignIn(authApp);
      const second = await startSignIn(authApp);

      expect(first.state).not.toBe(second.state);
    });
  });

  describe("GET /oauth/google/callback", () => {
    it("links Google to the existing account instead of duplicating it", async () => {
      const user = await createVerifiedUser(authApp, EMAIL, PASSWORD);
      stubGoogle(ANA_AT_GOOGLE);

      const response = await completeSignIn(authApp);

      expect(response.status).toBe(302);
      expect(response.headers.location).toBe(`${authApp.appOrigin}/app`);
      expect(cookieNamed(response, ACCESS_COOKIE)).toContain("HttpOnly");
      expect(cookieNamed(response, REFRESH_COOKIE)).toContain(
        "Path=/sessions/current/tokens",
      );
      expect(
        await authApp.dataSource
          .getRepository(User)
          .count({ where: { email: EMAIL } }),
      ).toBe(1);
      const google = await authApp.dataSource
        .getRepository(Account)
        .findOneOrFail({ where: { provider: Provider.Google } });
      expect(google.userId).toBe(user.id);
      expect(google.providerAccountId).toBe(GOOGLE_SUBJECT);
      expect(google.passwordHash).toBeNull();
      expect(
        authApp.dataSource
          .getMetadata(Account)
          .columns.map((column) => column.propertyName)
          .sort(),
      ).toEqual(ACCOUNT_COLUMNS);

      await authApp
        .post("/sessions")
        .send({ email: EMAIL, password: PASSWORD })
        .expect(201);
    });

    it("never stores the tokens Google returned", async () => {
      await createVerifiedUser(authApp, EMAIL, PASSWORD);
      stubGoogle(ANA_AT_GOOGLE);

      await completeSignIn(authApp);

      const rows = await authApp.dataSource.getRepository(Account).find();
      expect(JSON.stringify(rows)).not.toContain("google-access-token");
      expect(JSON.stringify(rows)).not.toContain("google-refresh-token");
      expect(JSON.stringify(rows)).not.toContain("google-id-token");
    });

    it("creates a verified user and a Google account for an unknown email", async () => {
      stubGoogle({ ...ANA_AT_GOOGLE, email: "Ana@Exemplo.com" });

      const response = await completeSignIn(authApp);

      expect(response.status).toBe(302);
      expect(response.headers.location).toBe(`${authApp.appOrigin}/app`);
      const user = await authApp.dataSource.getRepository(User).findOneOrFail({
        where: { email: EMAIL },
        relations: { accounts: true },
      });
      expect(user.emailVerified).toBe(true);
      expect(user.name).toBe("Ana Google");
      expect(user.accounts).toHaveLength(1);
      expect(user.accounts[0]?.provider).toBe(Provider.Google);
      expect(user.accounts[0]?.providerAccountId).toBe(GOOGLE_SUBJECT);
      expect(
        await authApp.dataSource
          .getRepository(Session)
          .count({ where: { userId: user.id } }),
      ).toBe(1);
    });

    it("signs the same Google account into the same user on the next visit", async () => {
      stubGoogle(ANA_AT_GOOGLE);
      await completeSignIn(authApp);
      stubGoogle({ ...ANA_AT_GOOGLE, email: "ana.nova@exemplo.com" });

      const response = await completeSignIn(authApp);

      expect(response.headers.location).toBe(`${authApp.appOrigin}/app`);
      expect(await authRowCounts(authApp)).toEqual([1, 1, 2]);
    });

    it("verifies an unverified account and drops the password someone else chose", async () => {
      await authApp
        .post("/users")
        .send({ name: "Intruso", email: EMAIL, password: PASSWORD })
        .expect(202);
      stubGoogle(ANA_AT_GOOGLE);

      const response = await completeSignIn(authApp);

      expect(response.headers.location).toBe(`${authApp.appOrigin}/app`);
      const user = await authApp.dataSource.getRepository(User).findOneOrFail({
        where: { email: EMAIL },
        relations: { accounts: true },
      });
      expect(user.emailVerified).toBe(true);
      expect(user.accounts.map((account) => account.provider)).toEqual([
        Provider.Google,
      ]);
      await authApp
        .post("/sessions")
        .send({ email: EMAIL, password: PASSWORD })
        .expect(401);
    });

    it("keeps the password of an account that was already verified", async () => {
      await createVerifiedUser(authApp, EMAIL, PASSWORD);
      stubGoogle(ANA_AT_GOOGLE);

      await completeSignIn(authApp);

      const providers = await authApp.dataSource
        .getRepository(Account)
        .find({ order: { provider: "ASC" } });
      expect(providers.map((account) => account.provider)).toEqual([
        Provider.Credential,
        Provider.Google,
      ]);
    });

    it("clears the state cookie so it cannot be replayed", async () => {
      stubGoogle(ANA_AT_GOOGLE);

      const response = await completeSignIn(authApp);

      const cleared = cookieNamed(response, STATE_COOKIE);
      expect(cleared).toContain("Path=/oauth/google");
      expect(valueOf(cleared ?? "")).toBe("");
    });

    it("refuses a state that differs from the cookie", async () => {
      stubGoogle(ANA_AT_GOOGLE);
      const started = await startSignIn(authApp);

      const response = await callback(
        authApp,
        { code: "google-code", state: `${started.state.slice(1)}A` },
        started.stateCookie,
      ).expect(302);

      expect(response.headers.location).toBe(
        `${authApp.appOrigin}/login?error=INVALID_STATE`,
      );
      expect(cookieNamed(response, ACCESS_COOKIE)).toBeUndefined();
      expect(await authRowCounts(authApp)).toEqual([0, 0, 0]);
    });

    it("refuses a state of a different length than the cookie", async () => {
      stubGoogle(ANA_AT_GOOGLE);
      const started = await startSignIn(authApp);

      const response = await callback(
        authApp,
        { code: "google-code", state: "short" },
        started.stateCookie,
      ).expect(302);

      expect(response.headers.location).toBe(
        `${authApp.appOrigin}/login?error=INVALID_STATE`,
      );
      expect(await authRowCounts(authApp)).toEqual([0, 0, 0]);
    });

    it("refuses a callback without the state cookie", async () => {
      stubGoogle(ANA_AT_GOOGLE);
      const started = await startSignIn(authApp);

      const response = await callback(authApp, {
        code: "google-code",
        state: started.state,
      }).expect(302);

      expect(response.headers.location).toBe(
        `${authApp.appOrigin}/login?error=INVALID_STATE`,
      );
      expect(await authRowCounts(authApp)).toEqual([0, 0, 0]);
    });

    it("refuses an email Google has not verified", async () => {
      stubGoogle({ ...ANA_AT_GOOGLE, email_verified: false });

      const response = await completeSignIn(authApp);

      expect(response.status).toBe(302);
      expect(response.headers.location).toBe(
        `${authApp.appOrigin}/login?error=UNVERIFIED_PROVIDER_EMAIL`,
      );
      expect(cookieNamed(response, ACCESS_COOKIE)).toBeUndefined();
      expect(await authRowCounts(authApp)).toEqual([0, 0, 0]);
    });

    it("sends a visitor who cancelled at Google back to the login", async () => {
      const started = await startSignIn(authApp);

      const response = await callback(
        authApp,
        { error: "access_denied", state: started.state },
        started.stateCookie,
      ).expect(302);

      expect(response.headers.location).toBe(
        `${authApp.appOrigin}/login?error=INVALID_STATE`,
      );
      expect(await authRowCounts(authApp)).toEqual([0, 0, 0]);
    });

    it("sends the visitor back to the login when Google refuses the code", async () => {
      nock(TOKEN_ORIGIN)
        .post(TOKEN_PATH)
        .reply(400, { error: "invalid_grant" });

      const response = await completeSignIn(authApp);

      expect(response.status).toBe(302);
      expect(response.headers.location).toBe(
        `${authApp.appOrigin}/login?error=INVALID_STATE`,
      );
      expect(await authRowCounts(authApp)).toEqual([0, 0, 0]);
    });

    it("never restarts the authorization from a callback without a code", async () => {
      const started = await startSignIn(authApp);

      const response = await callback(
        authApp,
        { state: started.state },
        started.stateCookie,
      ).expect(302);

      expect(response.headers.location).toBe(
        `${authApp.appOrigin}/login?error=INVALID_STATE`,
      );
      expect(cookiesOf(response).join()).not.toContain(ACCESS_COOKIE);
    });
  });
});
