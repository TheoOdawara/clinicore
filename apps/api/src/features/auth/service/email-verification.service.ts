import { Injectable } from "@nestjs/common";
import { BusinessError } from "../../../common/exceptions/business-error";
import { EnvironmentService } from "../../../core/config/environment.service";
import { MailService } from "../../../core/mail/mail.service";
import { verificationMail } from "../../../core/mail/templates";
import { EmailDispatchKind } from "../enums/email-dispatch-kind.enum";
import { SessionClient } from "../enums/session-client.enum";
import { VerificationPurpose } from "../enums/verification-purpose.enum";
import { EmailDispatchRepository } from "../repository/email-dispatch.repository";
import { UserRepository } from "../repository/user.repository";
import { VerificationRepository } from "../repository/verification.repository";
import { hashSecret } from "../utils/session-token";
import { issueToken } from "../utils/verification-token";
import { OpenSessionService, type OpenedSession } from "./open-session.service";

@Injectable()
export class EmailVerificationService {
  constructor(
    private readonly dispatches: EmailDispatchRepository,
    private readonly users: UserRepository,
    private readonly verifications: VerificationRepository,
    private readonly openSession: OpenSessionService,
    private readonly mail: MailService,
    private readonly environment: EnvironmentService,
  ) {}

  async request(email: string): Promise<void> {
    const address = email.toLowerCase();
    const accepted = await this.dispatches.registerDispatch(
      address,
      EmailDispatchKind.Verification,
    );

    if (!accepted) {
      return;
    }

    const user = await this.users.findByEmail(address);

    if (user === null || user.emailVerified) {
      return;
    }

    const issued = issueToken();
    await this.verifications.createToken(
      address,
      VerificationPurpose.EmailVerification,
      issued.token,
    );
    this.sendLink(address, user.name, issued.secret);
  }

  sendLink(email: string, name: string, secret: string): void {
    const link = `${this.environment.get("APP_ORIGIN")}/verify-email?token=${secret}`;

    this.mail.send(email, verificationMail(name, link));
  }

  async confirm(
    token: string,
    ipAddress: string | null,
    userAgent: string | null,
  ): Promise<OpenedSession> {
    const confirmation = await this.verifications.confirmEmail(
      hashSecret(token),
    );

    if (confirmation.status === "expired") {
      throw BusinessError.invalid("TOKEN_EXPIRED", "Token expired");
    }

    if (confirmation.status === "invalid") {
      throw BusinessError.invalid("INVALID_TOKEN", "Invalid token");
    }

    return this.openSession.open(
      confirmation.user,
      ipAddress,
      userAgent,
      SessionClient.Web,
    );
  }
}
