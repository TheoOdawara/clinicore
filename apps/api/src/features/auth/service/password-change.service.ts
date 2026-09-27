import { Injectable } from "@nestjs/common";
import { hash, verify } from "@node-rs/argon2";
import { BusinessError } from "../../../common/exceptions/business-error";
import { RevokedSessionRepository } from "../repository/revoked-session.repository";
import { UserRepository } from "../repository/user.repository";

@Injectable()
export class PasswordChangeService {
  constructor(
    private readonly users: UserRepository,
    private readonly revoked: RevokedSessionRepository,
  ) {}

  async change(
    userId: string,
    sessionId: string,
    currentPassword: string,
    newPassword: string,
  ): Promise<void> {
    const invalidPassword = (): BusinessError =>
      BusinessError.invalid("INVALID_PASSWORD", "Invalid password");
    const storedHash = await this.users.findCredentialHash(userId);

    if (storedHash === null || !(await verify(storedHash, currentPassword))) {
      throw invalidPassword();
    }

    const outcome = await this.users.changePassword(
      userId,
      sessionId,
      storedHash,
      await hash(newPassword),
    );

    if (outcome.status === "invalid") {
      throw invalidPassword();
    }

    await this.revoked.revokeMany(outcome.revokedSessionIds);
  }
}
