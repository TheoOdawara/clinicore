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

function stateCookieOptions(nodeEnv: Environment["NODE_ENV"]): CookieOptions {
  return {
    httpOnly: true,
    sameSite: "lax",
    secure: nodeEnv === "production",
    path: "/oauth/google",
  };
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

    const state = randomBytes(32).toString("base64url");
    request.res?.cookie(STATE_COOKIE, state, {
      ...stateCookieOptions(this.nodeEnv),
      maxAge: 600_000,
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

    const expectedBytes = Buffer.from(expected);
    const receivedBytes = Buffer.from(state);
    const sameState =
      expectedBytes.length === receivedBytes.length &&
      timingSafeEqual(expectedBytes, receivedBytes);

    done(null, sameState, state);
  }
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
      tokenURL: "https://oauth2.googleapis.com/token",
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

    const profileJson = json as Record<string, unknown>;
    const textField = (field: string): string | null => {
      const value = profileJson[field];

      if (typeof value !== "string" || value === "") {
        return null;
      }

      return value;
    };
    const nameOf = (email: string): string => {
      const name = textField("name") ?? email.slice(0, email.indexOf("@"));

      return name.slice(0, 100);
    };
    const imageOf = (): string | null => {
      const picture = textField("picture");

      if (picture === null || picture.length > 2048) {
        return null;
      }

      return picture;
    };
    const subject = textField("sub");
    const email = textField("email")?.toLowerCase() ?? null;

    if (subject === null || email === null) {
      throw new Error("Google returned a profile without sub or email");
    }

    return {
      subject,
      email,
      emailVerified: profileJson.email_verified === true,
      name: nameOf(email),
      image: imageOf(),
    };
  }
}
