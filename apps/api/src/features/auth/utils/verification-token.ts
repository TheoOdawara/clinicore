import { createSecret } from "./session-token";

const TOKEN_LIFETIME_IN_MILLISECONDS = 60 * 60 * 1000;

export interface IssuedToken {
  tokenHash: string;
  expiresAt: Date;
}

export function issueToken(): { secret: string; token: IssuedToken } {
  const generated = createSecret();

  return {
    secret: generated.secret,
    token: {
      tokenHash: generated.hash,
      expiresAt: new Date(Date.now() + TOKEN_LIFETIME_IN_MILLISECONDS),
    },
  };
}
