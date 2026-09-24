import { isUUID } from "class-validator";
import { createHash, randomBytes, timingSafeEqual } from "node:crypto";
import { SessionClient } from "../enums/session-client.enum";
import { REFRESH_MAX_AGE_IN_SECONDS } from "./session-cookies";

const SECRET_BYTES = 32;
const SEPARATOR = ".";
const MILLISECONDS = 1000;

export const REFRESH_TOKEN_FORMAT =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\.[A-Za-z0-9_-]{43}$/;

const SESSION_LIFETIME_IN_SECONDS: Record<SessionClient, number> = {
  [SessionClient.Web]: REFRESH_MAX_AGE_IN_SECONDS,
  [SessionClient.Mobile]: 604800,
};

export function sessionExpiresAt(client: SessionClient): Date {
  return new Date(
    Date.now() + SESSION_LIFETIME_IN_SECONDS[client] * MILLISECONDS,
  );
}

export interface GeneratedSecret {
  secret: string;
  hash: string;
}

export function hashSecret(secret: string): string {
  return createHash("sha256").update(secret).digest("hex");
}

export function createSecret(): GeneratedSecret {
  const secret = randomBytes(SECRET_BYTES).toString("base64url");

  return { secret, hash: hashSecret(secret) };
}

export function composeRefreshToken(sessionId: string, secret: string): string {
  return `${sessionId}${SEPARATOR}${secret}`;
}

export function parseRefreshToken(
  token: string,
): { sessionId: string; secret: string } | null {
  const separatorAt = token.indexOf(SEPARATOR);
  if (separatorAt <= 0 || separatorAt === token.length - 1) {
    return null;
  }

  const sessionId = token.slice(0, separatorAt);
  if (!isUUID(sessionId)) {
    return null;
  }

  return { sessionId, secret: token.slice(separatorAt + 1) };
}

export function hashesMatch(first: string, second: string): boolean {
  const firstBuffer = Buffer.from(first, "utf8");
  const secondBuffer = Buffer.from(second, "utf8");

  if (firstBuffer.length !== secondBuffer.length) {
    return false;
  }

  return timingSafeEqual(firstBuffer, secondBuffer);
}
