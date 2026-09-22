import { Injectable } from "@nestjs/common";
import { hash } from "@node-rs/argon2";
import { BusinessError } from "../../../common/exceptions/business-error";
import { EnvironmentService } from "../../../core/config/environment.service";
import { MailService } from "../../../core/mail/mail.service";
import { passwordResetMail } from "../../../core/mail/templates";
import { EmailDispatchKind } from "../enums/email-dispatch-kind.enum";
import { VerificationPurpose } from "../enums/verification-purpose.enum";
import { EmailDispatchRepository } from "../repository/email-dispatch.repository";
import { RevokedSessionRepository } from "../repository/revoked-session.repository";
import { UserRepository } from "../repository/user.repository";
import { VerificationRepository } from "../repository/verification.repository";
import { hashSecret } from "../utils/session-token";
import { issueToken } from "../utils/verification-token";

@Injectable()
export class PasswordResetService {
  constructor(
    private readonly dispatches: EmailDispatchRepository,
    private readonly users: UserRepository,
    private readonly verifications: VerificationRepository,
    private readonly revoked: RevokedSessionRepository,
    private readonly mail: MailService,
    private readonly environment: EnvironmentService,
  ) {}

  async request(email: string): Promise<void> {
    const address = email.toLowerCase();
    const accepted = await this.dispatches.registerDispatch(
      address,
      EmailDispatchKind.PasswordReset,
    );

    if (!accepted) {
      return;
    }

    const user = await this.users.findByEmail(address);

    if (user === null) {
      return;
    }

    const issued = issueToken();
    await this.verifications.createToken(
      address,
      VerificationPurpose.PasswordReset,
      issued.token,
    );
    const link = `${this.environment.get("APP_ORIGIN")}/reset-password?token=${issued.secret}`;
    this.mail.send(address, passwordResetMail(user.name, link));
  }

  async confirm(token: string, newPassword: string): Promise<void> {
    const outcome = await this.verifications.resetPassword(
      hashSecret(token),
      await hash(newPassword),
    );

    if (outcome.status === "invalid") {
      throw BusinessError.invalid("INVALID_TOKEN", "Invalid token");
    }

    await this.revoked.revokeMany(outcome.revokedSessionIds);
  }
}
