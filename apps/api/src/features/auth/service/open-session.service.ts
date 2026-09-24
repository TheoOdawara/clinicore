import { Injectable } from "@nestjs/common";
import { JwtService } from "@nestjs/jwt";
import { SessionUserResponse } from "../dto/session.response";
import type { User } from "../entities/user.entity";
import type { SessionClient } from "../enums/session-client.enum";
import { SessionRepository } from "../repository/session.repository";
import {
  composeRefreshToken,
  createSecret,
  sessionExpiresAt,
} from "../utils/session-token";
import { toSessionUser } from "./session.service";

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
    client: SessionClient,
  ): Promise<OpenedSession> {
    const refresh = createSecret();
    const session = await this.sessions.createSession(
      user.id,
      refresh.hash,
      sessionExpiresAt(client),
      ipAddress,
      userAgent,
      client,
    );

    return {
      user: toSessionUser(user),
      accessToken: this.jwt.sign({
        sub: user.id,
        sid: session.id,
        cli: client,
      }),
      refreshToken: composeRefreshToken(session.id, refresh.secret),
    };
  }
}
