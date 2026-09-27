import { Injectable } from "@nestjs/common";
import { JwtService } from "@nestjs/jwt";
import { BusinessError } from "../../../common/exceptions/business-error";
import { SessionRepository } from "../repository/session.repository";
import {
  composeRefreshToken,
  createSecret,
  hashSecret,
  hashesMatch,
  parseRefreshToken,
  sessionExpiresAt,
} from "../utils/session-token";

@Injectable()
export class RefreshSessionService {
  constructor(
    private readonly sessions: SessionRepository,
    private readonly jwt: JwtService,
  ) {}

  async refresh(
    presentedToken: string | undefined,
  ): Promise<{ accessToken: string; refreshToken: string }> {
    const parsed =
      presentedToken === undefined ? null : parseRefreshToken(presentedToken);

    if (parsed === null) {
      throw BusinessError.unauthorized("INVALID_SESSION", "Invalid session");
    }

    const presentedHash = hashSecret(parsed.secret);
    const next = createSecret();
    const outcome = await this.sessions.rotate(
      parsed.sessionId,
      (storedHash) => hashesMatch(storedHash, presentedHash),
      next.hash,
      sessionExpiresAt,
    );

    if (outcome.status === "invalid") {
      throw BusinessError.unauthorized("INVALID_SESSION", "Invalid session");
    }

    if (outcome.status === "reused") {
      throw BusinessError.unauthorized(
        "SESSION_REUSED",
        "Refresh token reuse detected",
      );
    }

    return {
      accessToken: this.jwt.sign({
        sub: outcome.session.userId,
        sid: outcome.session.id,
        cli: outcome.session.client,
      }),
      refreshToken: composeRefreshToken(outcome.session.id, next.secret),
    };
  }
}
