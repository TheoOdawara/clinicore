import { Injectable } from "@nestjs/common";
import { PassportStrategy } from "@nestjs/passport";
import type { Request } from "express";
import { ExtractJwt, Strategy } from "passport-jwt";
import { BusinessError } from "../../../common/exceptions/business-error";
import type { AuthenticatedUser } from "../../../common/decorators/current-user.decorator";
import { EnvironmentService } from "../../../core/config/environment.service";
import { RevokedSessionRepository } from "../repository/revoked-session.repository";
import { ACCESS_COOKIE } from "../utils/session-cookies";

interface AccessTokenPayload {
  sub: string;
  sid: string;
}

function fromAccessCookie(request: Request): string | null {
  const cookies = request.cookies as Record<string, string> | undefined;

  return cookies?.[ACCESS_COOKIE] ?? null;
}

@Injectable()
export class JwtStrategy extends PassportStrategy(Strategy) {
  constructor(
    environment: EnvironmentService,
    private readonly revoked: RevokedSessionRepository,
  ) {
    super({
      jwtFromRequest: ExtractJwt.fromExtractors([fromAccessCookie]),
      secretOrKey: environment.get("JWT_SECRET"),
      ignoreExpiration: false,
    });
  }

  async validate(payload: AccessTokenPayload): Promise<AuthenticatedUser> {
    if (await this.revoked.isRevoked(payload.sid)) {
      throw BusinessError.unauthorized("INVALID_SESSION", "Invalid session");
    }

    return { userId: payload.sub, sessionId: payload.sid };
  }
}
