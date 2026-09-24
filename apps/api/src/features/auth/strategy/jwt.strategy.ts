import { Injectable } from "@nestjs/common";
import { PassportStrategy } from "@nestjs/passport";
import type { Request } from "express";
import { ExtractJwt, Strategy } from "passport-jwt";
import { BusinessError } from "../../../common/exceptions/business-error";
import {
  sessionClientOf,
  type SessionClientKind,
} from "../../../common/decorators/session-client.decorator";
import type { AuthenticatedUser } from "../../../common/decorators/current-user.decorator";
import { EnvironmentService } from "../../../core/config/environment.service";
import { RevokedSessionRepository } from "../repository/revoked-session.repository";
import { ACCESS_COOKIE } from "../utils/session-cookies";

interface AccessTokenPayload {
  sub: string;
  sid: string;
  cli: SessionClientKind;
}

const fromBearerHeader = ExtractJwt.fromAuthHeaderAsBearerToken();

function fromAccessCookie(request: Request): string | null {
  const cookies = request.cookies as Record<string, string> | undefined;

  return cookies?.[ACCESS_COOKIE] ?? null;
}

function fromDeclaredTransport(request: Request): string | null {
  if (sessionClientOf(request) === "mobile") {
    return fromBearerHeader(request);
  }

  return fromAccessCookie(request);
}

@Injectable()
export class JwtStrategy extends PassportStrategy(Strategy) {
  constructor(
    environment: EnvironmentService,
    private readonly revoked: RevokedSessionRepository,
  ) {
    super({
      jwtFromRequest: fromDeclaredTransport,
      secretOrKey: environment.get("JWT_SECRET"),
      ignoreExpiration: false,
      passReqToCallback: true,
    });
  }

  async validate(
    request: Request,
    payload: AccessTokenPayload,
  ): Promise<AuthenticatedUser> {
    if (payload.cli !== sessionClientOf(request)) {
      throw BusinessError.unauthorized("INVALID_SESSION", "Invalid session");
    }

    if (await this.revoked.isRevoked(payload.sid)) {
      throw BusinessError.unauthorized("INVALID_SESSION", "Invalid session");
    }

    return { userId: payload.sub, sessionId: payload.sid };
  }
}
