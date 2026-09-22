import { Injectable } from "@nestjs/common";
import { JwtService } from "@nestjs/jwt";
import { SessionUserResponse } from "../dto/session.response";
import type { User } from "../entities/user.entity";
import { SessionRepository } from "../repository/session.repository";
import { REFRESH_MAX_AGE_IN_SECONDS } from "../utils/session-cookies";
import { composeRefreshToken, createSecret } from "../utils/session-token";
import { toSessionUser } from "./session.service";

const MILLISECONDS = 1000;

export interface OpenedSession {
  user: SessionUserResponse;
  accessToken: string;
  refreshToken: string;
}

@Injectable()
export class OpenSessionService {
  constructor(
    private readonly sessions: SessionRepository,
    private readonly jwt: JwtService,
  ) {}

  async open(
    user: User,
    ipAddress: string | null,
    userAgent: string | null,
  ): Promise<OpenedSession> {
    const refresh = createSecret();
    const expiresAt = new Date(
      Date.now() + REFRESH_MAX_AGE_IN_SECONDS * MILLISECONDS,
    );
    const session = await this.sessions.createSession(
      user.id,
      refresh.hash,
      expiresAt,
      ipAddress,
      userAgent,
    );

    return {
      user: toSessionUser(user),
      accessToken: this.jwt.sign({ sub: user.id, sid: session.id }),
      refreshToken: composeRefreshToken(session.id, refresh.secret),
    };
  }
}
