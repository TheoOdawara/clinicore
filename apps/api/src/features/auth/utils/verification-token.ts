import { createSecret } from "./session-token";

export interface IssuedToken {
  tokenHash: string;
  expiresAt: Date;
}

export function issueToken(): { secret: string; token: IssuedToken } {
  const lifetimeInMilliseconds = 60 * 60 * 1000;
  const generated = createSecret();

  return {
    secret: generated.secret,
    token: {
      tokenHash: generated.hash,
      expiresAt: new Date(Date.now() + lifetimeInMilliseconds),
    },
  };
}
