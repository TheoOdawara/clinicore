import { Injectable } from "@nestjs/common";
import { JwtService } from "@nestjs/jwt";
import { BusinessError } from "../../../common/exceptions/business-error";
import { RevokedSessionRepository } from "../repository/revoked-session.repository";
import { SessionRepository } from "../repository/session.repository";
import { REFRESH_MAX_AGE_IN_SECONDS } from "../utils/session-cookies";
import {
  composeRefreshToken,
  createRefreshSecret,
  hashSecret,
  hashesMatch,
  parseRefreshToken,
} from "../utils/session-token";

const MILLISECONDS = 1000;

export interface RotatedSession {
  accessToken: string;
  refreshToken: string;
}

@Injectable()
export class RefreshSessionService {
  constructor(
    private readonly sessions: SessionRepository,
    private readonly revoked: RevokedSessionRepository,
    private readonly jwt: JwtService,
  ) {}

  async refresh(presentedToken: string | undefined): Promise<RotatedSession> {
    const parsed =
      presentedToken === undefined ? null : parseRefreshToken(presentedToken);

    if (parsed === null) {
      throw BusinessError.unauthorized("INVALID_SESSION", "Invalid session");
    }

    const presentedHash = hashSecret(parsed.secret);
    const next = createRefreshSecret();
    const expiresAt = new Date(
      Date.now() + REFRESH_MAX_AGE_IN_SECONDS * MILLISECONDS,
    );
    const outcome = await this.sessions.rotate(
      parsed.sessionId,
      (storedHash) => hashesMatch(storedHash, presentedHash),
      next.hash,
      expiresAt,
    );

    if (outcome.status === "invalid") {
      throw BusinessError.unauthorized("INVALID_SESSION", "Invalid session");
    }

    if (outcome.status === "reused") {
      await this.revoked.revoke(parsed.sessionId);
      throw BusinessError.unauthorized(
        "SESSION_REUSED",
        "Refresh token reuse detected",
      );
    }

    return {
      accessToken: this.jwt.sign({
        sub: outcome.session.userId,
        sid: outcome.session.id,
      }),
      refreshToken: composeRefreshToken(outcome.session.id, next.secret),
    };
  }
}
