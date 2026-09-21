import { Injectable } from "@nestjs/common";
import { BusinessError } from "../../../common/exceptions/business-error";
import { SessionRepository } from "../repository/session.repository";

@Injectable()
export class SignOutService {
  constructor(private readonly sessions: SessionRepository) {}

  async signOut(sessionId: string): Promise<void> {
    const revoked = await this.sessions.revokeSession(sessionId);

    if (!revoked) {
      throw BusinessError.unauthorized("INVALID_SESSION", "Invalid session");
    }
  }
}
