import type { CookieOptions, Response } from "express";
import type { Environment } from "../../../core/config/env.validation";

export const ACCESS_COOKIE = "clinicore_access";
export const REFRESH_COOKIE = "clinicore_refresh";
export const ACCESS_MAX_AGE_IN_SECONDS = 900;
export const REFRESH_MAX_AGE_IN_SECONDS = 86400;
const REFRESH_PATH = "/sessions/current/tokens";

function baseOptions(nodeEnv: Environment["NODE_ENV"]): CookieOptions {
  return {
    httpOnly: true,
    sameSite: "lax",
    secure: nodeEnv === "production",
  };
}

export function setSessionCookies(
  response: Response,
  nodeEnv: Environment["NODE_ENV"],
  tokens: { accessToken: string; refreshToken: string },
): void {
  const milliseconds = 1000;

  response.cookie(ACCESS_COOKIE, tokens.accessToken, {
    ...baseOptions(nodeEnv),
    path: "/",
    maxAge: ACCESS_MAX_AGE_IN_SECONDS * milliseconds,
  });
  response.cookie(REFRESH_COOKIE, tokens.refreshToken, {
    ...baseOptions(nodeEnv),
    path: REFRESH_PATH,
    maxAge: REFRESH_MAX_AGE_IN_SECONDS * milliseconds,
  });
}

export function clearSessionCookies(
  response: Response,
  nodeEnv: Environment["NODE_ENV"],
): void {
  response.cookie(ACCESS_COOKIE, "", {
    ...baseOptions(nodeEnv),
    path: "/",
    maxAge: 0,
  });
  response.cookie(REFRESH_COOKIE, "", {
    ...baseOptions(nodeEnv),
    path: REFRESH_PATH,
    maxAge: 0,
  });
}
