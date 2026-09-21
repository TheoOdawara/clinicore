import { Inject, Injectable } from "@nestjs/common";
import { JwtService } from "@nestjs/jwt";
import { verify } from "@node-rs/argon2";
import { BusinessError } from "../../../common/exceptions/business-error";
import type { User } from "../entities/user.entity";
import { RevokedSessionRepository } from "../repository/revoked-session.repository";
import { SessionRepository } from "../repository/session.repository";
import { UserRepository } from "../repository/user.repository";
import { REFRESH_MAX_AGE_IN_SECONDS } from "../utils/session-cookies";
import {
  composeRefreshToken,
  createRefreshSecret,
} from "../utils/session-token";
import { DUMMY_PASSWORD_HASH } from "./dummy-password-hash";
import { SessionUserResponse } from "../dto/session.response";
import { toSessionUser } from "./session.service";

const MILLISECONDS = 1000;

export interface OpenedSession {
  user: SessionUserResponse;
  accessToken: string;
  refreshToken: string;
}

@Injectable()
export class SignInService {
  constructor(
    private readonly users: UserRepository,
    private readonly sessions: SessionRepository,
    private readonly revoked: RevokedSessionRepository,
    private readonly jwt: JwtService,
    @Inject(DUMMY_PASSWORD_HASH) private readonly dummyHash: string,
  ) {}

  async signIn(
    email: string,
    password: string,
    ipAddress: string | null,
    userAgent: string | null,
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
      throw BusinessError.forbidden("EMAIL_NOT_VERIFIED", "Email not verified");
    }

    return this.openSession(found.user, ipAddress, userAgent);
  }

  private async openSession(
    user: User,
    ipAddress: string | null,
    userAgent: string | null,
  ): Promise<OpenedSession> {
    const refresh = createRefreshSecret();
    const expiresAt = new Date(
      Date.now() + REFRESH_MAX_AGE_IN_SECONDS * MILLISECONDS,
    );
    const created = await this.sessions.createSession(
      user.id,
      refresh.hash,
      expiresAt,
      ipAddress,
      userAgent,
    );

    await this.revoked.revokeMany(created.revokedIds);

    return {
      user: toSessionUser(user),
      accessToken: this.jwt.sign({ sub: user.id, sid: created.session.id }),
      refreshToken: composeRefreshToken(created.session.id, refresh.secret),
    };
  }
}
