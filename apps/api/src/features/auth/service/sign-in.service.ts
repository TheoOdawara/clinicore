import { Inject, Injectable } from "@nestjs/common";
import { verify } from "@node-rs/argon2";
import { BusinessError } from "../../../common/exceptions/business-error";
import type { SessionClient } from "../enums/session-client.enum";
import { UserRepository } from "../repository/user.repository";
import { DUMMY_PASSWORD_HASH } from "./dummy-password-hash";
import { EmailVerificationService } from "./email-verification.service";
import { OpenSessionService, type OpenedSession } from "./open-session.service";

@Injectable()
export class SignInService {
  constructor(
    private readonly users: UserRepository,
    private readonly openSession: OpenSessionService,
    private readonly emailVerification: EmailVerificationService,
    @Inject(DUMMY_PASSWORD_HASH) private readonly dummyHash: string,
  ) {}

  async signIn(
    email: string,
    password: string,
    ipAddress: string | null,
    userAgent: string | null,
    client: SessionClient,
  ): Promise<OpenedSession> {
    const found = await this.users.findByEmailWithCredentialAccount(
      email.toLowerCase(),
    );
    const account = found?.account ?? null;
    const storedHash = account?.passwordHash ?? this.dummyHash;
    const passwordMatches = await verify(storedHash, password);

    if (found === null || account === null || !passwordMatches) {
      throw BusinessError.unauthorized(
        "INVALID_CREDENTIALS",
        "Invalid email or password",
      );
    }

    if (!found.user.emailVerified) {
      await this.emailVerification.request(found.user.email);
      throw BusinessError.forbidden("EMAIL_NOT_VERIFIED", "Email not verified");
    }

    return this.openSession.open(found.user, ipAddress, userAgent, client);
  }
}
