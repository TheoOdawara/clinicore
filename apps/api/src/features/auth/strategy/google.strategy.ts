import { Injectable } from "@nestjs/common";
import { PassportStrategy } from "@nestjs/passport";
import type { CookieOptions, Request } from "express";
import { randomBytes, timingSafeEqual } from "node:crypto";
import type {
  Metadata,
  StateStore,
  StateStoreStoreCallback,
  StateStoreVerifyCallback,
} from "passport-oauth2";
import { Strategy, type Profile } from "passport-google-oauth20";
import { EnvironmentService } from "../../../core/config/environment.service";
import type { Environment } from "../../../core/config/env.validation";
import type { GoogleIdentity } from "../service/google-account.service";

const GOOGLE_STRATEGY = "google";
const STATE_COOKIE = "clinicore_oauth_state";
const STATE_PATH = "/oauth/google";
const STATE_MAX_AGE_IN_MILLISECONDS = 600_000;
const STATE_BYTES = 32;
const GOOGLE_TOKEN_URL = "https://oauth2.googleapis.com/token";
const NAME_LIMIT = 100;
const IMAGE_LIMIT = 2048;

function stateCookieOptions(nodeEnv: Environment["NODE_ENV"]): CookieOptions {
  return {
    httpOnly: true,
    sameSite: "lax",
    secure: nodeEnv === "production",
    path: STATE_PATH,
  };
}

function sameState(expected: string, received: string): boolean {
  const expectedBytes = Buffer.from(expected);
  const receivedBytes = Buffer.from(received);

  if (expectedBytes.length !== receivedBytes.length) {
    return false;
  }

  return timingSafeEqual(expectedBytes, receivedBytes);
}

class CookieStateStore implements StateStore {
  constructor(private readonly nodeEnv: Environment["NODE_ENV"]) {}

  store(request: Request, callback: StateStoreStoreCallback): void;
  store(
    request: Request,
    meta: Metadata,
    callback: StateStoreStoreCallback,
  ): void;
  store(
    request: Request,
    metaOrCallback: Metadata | StateStoreStoreCallback,
    callback?: StateStoreStoreCallback,
  ): void {
    const done = callback ?? metaOrCallback;
    if (typeof done !== "function") {
      throw new Error("the OAuth state store was called without a callback");
    }

    const state = randomBytes(STATE_BYTES).toString("base64url");
    request.res?.cookie(STATE_COOKIE, state, {
      ...stateCookieOptions(this.nodeEnv),
      maxAge: STATE_MAX_AGE_IN_MILLISECONDS,
    });
    done(null, state);
  }

  verify(
    request: Request,
    state: string | undefined,
    callback: StateStoreVerifyCallback,
  ): void;
  verify(
    request: Request,
    state: string | undefined,
    meta: Metadata,
    callback: StateStoreVerifyCallback,
  ): void;
  verify(
    request: Request,
    state: string | undefined,
    metaOrCallback: Metadata | StateStoreVerifyCallback,
    callback?: StateStoreVerifyCallback,
  ): void {
    const done = callback ?? metaOrCallback;
    if (typeof done !== "function") {
      throw new Error("the OAuth state store was called without a callback");
    }

    const cookies = request.cookies as Record<string, string> | undefined;
    const expected = cookies?.[STATE_COOKIE];
    request.res?.clearCookie(STATE_COOKIE, stateCookieOptions(this.nodeEnv));

    if (expected === undefined || state === undefined) {
      done(null, false, state);
      return;
    }

    done(null, sameState(expected, state), state);
  }
}

function textField(
  json: Record<string, unknown>,
  field: string,
): string | null {
  const value = json[field];

  if (typeof value !== "string" || value === "") {
    return null;
  }

  return value;
}

function nameOf(json: Record<string, unknown>, email: string): string {
  const name = textField(json, "name") ?? email.slice(0, email.indexOf("@"));

  return name.slice(0, NAME_LIMIT);
}

function imageOf(json: Record<string, unknown>): string | null {
  const picture = textField(json, "picture");

  if (picture === null || picture.length > IMAGE_LIMIT) {
    return null;
  }

  return picture;
}

function identityOf(json: Record<string, unknown>): GoogleIdentity {
  const subject = textField(json, "sub");
  const email = textField(json, "email")?.toLowerCase() ?? null;

  if (subject === null || email === null) {
    throw new Error("Google returned a profile without sub or email");
  }

  return {
    subject,
    email,
    emailVerified: json.email_verified === true,
    name: nameOf(json, email),
    image: imageOf(json),
  };
}

@Injectable()
export class GoogleStrategy extends PassportStrategy(
  Strategy,
  GOOGLE_STRATEGY,
) {
  constructor(environment: EnvironmentService) {
    super({
      clientID: environment.get("GOOGLE_CLIENT_ID"),
      clientSecret: environment.get("GOOGLE_CLIENT_SECRET"),
      callbackURL: `${environment.get("API_URL")}/oauth/google/callback`,
      tokenURL: GOOGLE_TOKEN_URL,
      scope: ["openid", "email", "profile"],
      passReqToCallback: false,
      store: new CookieStateStore(environment.get("NODE_ENV")),
    });
  }

  validate(
    _accessToken: string,
    _refreshToken: string,
    profile: Profile,
  ): GoogleIdentity {
    const json: unknown = JSON.parse(profile._raw);

    if (typeof json !== "object" || json === null) {
      throw new Error("Google returned a profile that is not an object");
    }

    return identityOf(json as Record<string, unknown>);
  }
}
