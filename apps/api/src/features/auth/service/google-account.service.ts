import { Injectable } from "@nestjs/common";
import { BusinessError } from "../../../common/exceptions/business-error";
import type { SessionClient } from "../enums/session-client.enum";
import { UserRepository } from "../repository/user.repository";
import { OpenSessionService, type OpenedSession } from "./open-session.service";

export interface GoogleIdentity {
  subject: string;
  email: string;
  emailVerified: boolean;
  name: string;
  image: string | null;
}

@Injectable()
export class GoogleAccountService {
  constructor(
    private readonly users: UserRepository,
    private readonly openSession: OpenSessionService,
  ) {}

  async signIn(
    identity: GoogleIdentity,
    ipAddress: string | null,
    userAgent: string | null,
    client: SessionClient,
  ): Promise<OpenedSession> {
    if (!identity.emailVerified) {
      throw BusinessError.forbidden(
        "UNVERIFIED_PROVIDER_EMAIL",
        "Google email not verified",
      );
    }

    const user = await this.users.linkOrCreateGoogleAccount(identity);

    return this.openSession.open(user, ipAddress, userAgent, client);
  }
}
