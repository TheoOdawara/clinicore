import { Injectable } from "@nestjs/common";
import { BusinessError } from "../../../common/exceptions/business-error";
import { RevokedSessionRepository } from "../repository/revoked-session.repository";
import { SessionRepository } from "../repository/session.repository";

@Injectable()
export class SignOutService {
  constructor(
    private readonly sessions: SessionRepository,
    private readonly revoked: RevokedSessionRepository,
  ) {}

  async signOut(sessionId: string): Promise<void> {
    const deleted = await this.sessions.deleteById(sessionId);

    if (!deleted) {
      throw BusinessError.unauthorized("INVALID_SESSION", "Invalid session");
    }

    await this.revoked.revoke(sessionId);
  }
}
