import {
  Body,
  Controller,
  Delete,
  Get,
  Header,
  HttpCode,
  HttpStatus,
  Post,
  Req,
  Res,
} from "@nestjs/common";
import {
  ApiAcceptedResponse,
  ApiCreatedResponse,
  ApiHeader,
  ApiNoContentResponse,
  ApiOkResponse,
  ApiOperation,
  ApiTags,
} from "@nestjs/swagger";
import { Throttle } from "@nestjs/throttler";
import type { Request, Response } from "express";
import {
  CurrentUser,
  type AuthenticatedUser,
} from "../../../common/decorators/current-user.decorator";
import { ApiProblemResponse } from "../../../common/decorators/api-problem-response.decorator";
import { Public } from "../../../common/decorators/public.decorator";
import {
  SESSION_CLIENT_HEADER,
  SessionClient,
  type SessionClientKind,
} from "../../../common/decorators/session-client.decorator";
import { BusinessError } from "../../../common/exceptions/business-error";
import { EnvironmentService } from "../../../core/config/environment.service";
import { EmailDto } from "../dto/email.dto";
import { PasswordResetConfirmationDto } from "../dto/password-reset-confirmation.dto";
import { RefreshTokenDto } from "../dto/refresh-token.dto";
import {
  RefreshResponse,
  SessionResponse,
  SessionTokensResponse,
} from "../dto/session.response";
import { SignInDto } from "../dto/sign-in.dto";
import { SignUpDto } from "../dto/sign-up.dto";
import { TokenDto } from "../dto/token.dto";
import { EmailVerificationService } from "../service/email-verification.service";
import { PasswordResetService } from "../service/password-reset.service";
import { RefreshSessionService } from "../service/refresh-session.service";
import { SessionService } from "../service/session.service";
import { SignInService } from "../service/sign-in.service";
import { SignOutService } from "../service/sign-out.service";
import { SignUpService } from "../service/sign-up.service";
import { SESSION_CLIENT_BY_KIND } from "../enums/session-client.enum";
import {
  ACCESS_MAX_AGE_IN_SECONDS,
  REFRESH_COOKIE,
  clearSessionCookies,
  setSessionCookies,
} from "../utils/session-cookies";

const USER_AGENT_LIMIT = 512;
const ONE_MINUTE = 60_000;
const REDIS_UNAVAILABLE = "SERVICE_UNAVAILABLE when Redis is unreachable";
const CURRENT_SESSION_PATH = "/sessions/current";
const SESSION_COOKIES = {
  "Set-Cookie": {
    description:
      "clinicore_access on Path=/ for 900s, and clinicore_refresh on Path=/sessions/current/tokens for 86400s; both HttpOnly and SameSite=Lax",
    schema: { type: "array", items: { type: "string" } },
  },
} as const;

function userAgentOf(request: Request): string | null {
  const userAgent = request.headers["user-agent"];

  if (userAgent === undefined) {
    return null;
  }

  return userAgent.slice(0, USER_AGENT_LIMIT);
}

function refreshTokenOf(request: Request): string | undefined {
  const cookies = request.cookies as Record<string, string> | undefined;

  return cookies?.[REFRESH_COOKIE];
}

function tokensOf(session: {
  accessToken: string;
  refreshToken: string;
}): SessionTokensResponse {
  return {
    accessToken: session.accessToken,
    refreshToken: session.refreshToken,
    accessTokenExpiresIn: ACCESS_MAX_AGE_IN_SECONDS,
  };
}

@ApiTags("Auth")
@ApiHeader({
  name: SESSION_CLIENT_HEADER,
  required: false,
  enum: ["mobile"],
  description:
    "mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 INVALID_CLIENT",
})
@ApiProblemResponse(HttpStatus.TOO_MANY_REQUESTS, "RATE_LIMITED")
@Controller()
export class AuthController {
  constructor(
    private readonly signUpService: SignUpService,
    private readonly signInService: SignInService,
    private readonly refreshSessionService: RefreshSessionService,
    private readonly signOutService: SignOutService,
    private readonly sessionService: SessionService,
    private readonly emailVerificationService: EmailVerificationService,
    private readonly passwordResetService: PasswordResetService,
    private readonly environment: EnvironmentService,
  ) {}

  @Public()
  @Post("users")
  @Throttle({ default: { ttl: ONE_MINUTE, limit: 3 } })
  @HttpCode(HttpStatus.ACCEPTED)
  @ApiOperation({
    summary: "Register an email and password account",
    description:
      "Always answers 202 with an empty body, whether the email is new or already registered.",
  })
  @ApiAcceptedResponse({ description: "Accepted, with no body" })
  @ApiProblemResponse(
    HttpStatus.BAD_REQUEST,
    "VALIDATION_FAILED, with the offending field in fields",
  )
  @ApiProblemResponse(HttpStatus.FORBIDDEN, "INVALID_ORIGIN")
  @ApiProblemResponse(HttpStatus.SERVICE_UNAVAILABLE, REDIS_UNAVAILABLE)
  async signUp(@Body() body: SignUpDto): Promise<void> {
    await this.signUpService.signUp(body.name, body.email, body.password);
  }

  @Public()
  @Post("sessions")
  @Throttle({ default: { ttl: ONE_MINUTE, limit: 5 } })
  @HttpCode(HttpStatus.CREATED)
  @Header("Location", CURRENT_SESSION_PATH)
  @ApiOperation({
    summary: "Open a session",
    description:
      "Issues the session cookies, or the tokens in the body with Clinicore-Client: mobile.",
  })
  @ApiCreatedResponse({
    description: "The signed-in user, and the tokens for the mobile app",
    type: SessionResponse,
    headers: {
      ...SESSION_COOKIES,
      Location: {
        description: "The session that was opened",
        schema: { type: "string", example: CURRENT_SESSION_PATH },
      },
    },
  })
  @ApiProblemResponse(
    HttpStatus.BAD_REQUEST,
    "VALIDATION_FAILED, or INVALID_CLIENT",
  )
  @ApiProblemResponse(
    HttpStatus.UNAUTHORIZED,
    "INVALID_CREDENTIALS, for both a wrong password and an unknown email",
  )
  @ApiProblemResponse(
    HttpStatus.FORBIDDEN,
    "EMAIL_NOT_VERIFIED, or INVALID_ORIGIN",
  )
  @ApiProblemResponse(HttpStatus.SERVICE_UNAVAILABLE, REDIS_UNAVAILABLE)
  async signIn(
    @Body() body: SignInDto,
    @SessionClient() client: SessionClientKind,
    @Req() request: Request,
    @Res({ passthrough: true }) response: Response,
  ): Promise<SessionResponse> {
    const opened = await this.signInService.signIn(
      body.email,
      body.password,
      request.ip ?? null,
      userAgentOf(request),
      SESSION_CLIENT_BY_KIND[client],
    );

    if (client === "mobile") {
      return { user: opened.user, tokens: tokensOf(opened) };
    }

    setSessionCookies(response, this.environment.get("NODE_ENV"), opened);

    return { user: opened.user };
  }

  @Public()
  @Post("sessions/current/tokens")
  @Throttle({ default: { ttl: ONE_MINUTE, limit: 30 } })
  @HttpCode(HttpStatus.NO_CONTENT)
  @ApiOperation({
    summary: "Rotate the refresh token",
    description:
      "Authenticated by the clinicore_refresh cookie, or by refreshToken in the body with Clinicore-Client: mobile. Presenting a token that was already rotated drops the whole session.",
  })
  @ApiNoContentResponse({
    description: "Rotated, with both cookies replaced",
    headers: SESSION_COOKIES,
  })
  @ApiOkResponse({
    description: "Rotated for the mobile app, with the new tokens",
    type: RefreshResponse,
  })
  @ApiProblemResponse(
    HttpStatus.BAD_REQUEST,
    "VALIDATION_FAILED at #/refreshToken, or INVALID_CLIENT",
  )
  @ApiProblemResponse(
    HttpStatus.UNAUTHORIZED,
    "INVALID_SESSION, or SESSION_REUSED when the old token comes back",
  )
  @ApiProblemResponse(HttpStatus.FORBIDDEN, "INVALID_ORIGIN")
  @ApiProblemResponse(HttpStatus.SERVICE_UNAVAILABLE, REDIS_UNAVAILABLE)
  async refresh(
    @Body() body: RefreshTokenDto,
    @SessionClient() client: SessionClientKind,
    @Req() request: Request,
    @Res({ passthrough: true }) response: Response,
  ): Promise<RefreshResponse | undefined> {
    if (client === "mobile") {
      if (body.refreshToken === undefined) {
        throw BusinessError.invalid("VALIDATION_FAILED", "Validation failed", [
          { pointer: "#/refreshToken", code: "IS_DEFINED" },
        ]);
      }

      const rotated = await this.refreshSessionService.refresh(
        body.refreshToken,
      );
      response.status(HttpStatus.OK);

      return { tokens: tokensOf(rotated) };
    }

    const rotated = await this.refreshSessionService.refresh(
      refreshTokenOf(request),
    );
    setSessionCookies(response, this.environment.get("NODE_ENV"), rotated);

    return undefined;
  }

  @Delete("sessions/current")
  @HttpCode(HttpStatus.NO_CONTENT)
  @ApiOperation({
    summary: "Close the current session",
    description:
      "Deletes the session row and denies the access token that is still inside its 15 minutes.",
  })
  @ApiNoContentResponse({
    description:
      "Signed out, with both cookies expired; no cookie for the mobile app",
    headers: SESSION_COOKIES,
  })
  @ApiProblemResponse(HttpStatus.UNAUTHORIZED, "INVALID_SESSION")
  @ApiProblemResponse(HttpStatus.FORBIDDEN, "INVALID_ORIGIN")
  @ApiProblemResponse(
    HttpStatus.SERVICE_UNAVAILABLE,
    "SERVICE_UNAVAILABLE when Redis is unreachable; the session survives",
  )
  async signOut(
    @CurrentUser() user: AuthenticatedUser,
    @SessionClient() client: SessionClientKind,
    @Res({ passthrough: true }) response: Response,
  ): Promise<void> {
    await this.signOutService.signOut(user.sessionId);

    if (client === "mobile") {
      return;
    }

    clearSessionCookies(response, this.environment.get("NODE_ENV"));
  }

  @Get("sessions/current")
  @ApiOperation({ summary: "Read the signed-in user" })
  @ApiOkResponse({ description: "The signed-in user", type: SessionResponse })
  @ApiProblemResponse(HttpStatus.UNAUTHORIZED, "INVALID_SESSION")
  @ApiProblemResponse(HttpStatus.SERVICE_UNAVAILABLE, REDIS_UNAVAILABLE)
  async session(
    @CurrentUser() user: AuthenticatedUser,
  ): Promise<SessionResponse> {
    return { user: await this.sessionService.currentUser(user.userId) };
  }

  @Public()
  @Post("email-verifications")
  @Throttle({ default: { ttl: ONE_MINUTE, limit: 3 } })
  @HttpCode(HttpStatus.ACCEPTED)
  @ApiOperation({
    summary: "Send a new email verification link",
    description:
      "Always answers 202 with an empty body. At most one link per address every 60 seconds and five every 24 hours.",
  })
  @ApiAcceptedResponse({ description: "Accepted, with no body" })
  @ApiProblemResponse(HttpStatus.BAD_REQUEST, "VALIDATION_FAILED")
  @ApiProblemResponse(HttpStatus.FORBIDDEN, "INVALID_ORIGIN")
  @ApiProblemResponse(HttpStatus.SERVICE_UNAVAILABLE, REDIS_UNAVAILABLE)
  async requestEmailVerification(@Body() body: EmailDto): Promise<void> {
    await this.emailVerificationService.request(body.email);
  }

  @Public()
  @Post("email-verifications/confirmation")
  @HttpCode(HttpStatus.NO_CONTENT)
  @ApiOperation({
    summary: "Confirm the email and open a session",
    description:
      "Consumes every pending verification token of the address and issues the session cookies.",
  })
  @ApiNoContentResponse({
    description: "Verified and signed in",
    headers: SESSION_COOKIES,
  })
  @ApiProblemResponse(
    HttpStatus.BAD_REQUEST,
    "INVALID_TOKEN, TOKEN_EXPIRED, or VALIDATION_FAILED",
  )
  @ApiProblemResponse(HttpStatus.FORBIDDEN, "INVALID_ORIGIN")
  @ApiProblemResponse(HttpStatus.SERVICE_UNAVAILABLE, REDIS_UNAVAILABLE)
  async confirmEmailVerification(
    @Body() body: TokenDto,
    @Req() request: Request,
    @Res({ passthrough: true }) response: Response,
  ): Promise<void> {
    const opened = await this.emailVerificationService.confirm(
      body.token,
      request.ip ?? null,
      userAgentOf(request),
    );

    setSessionCookies(response, this.environment.get("NODE_ENV"), opened);
  }

  @Public()
  @Post("password-resets")
  @Throttle({ default: { ttl: ONE_MINUTE, limit: 5 } })
  @HttpCode(HttpStatus.ACCEPTED)
  @ApiOperation({
    summary: "Send a password reset link",
    description:
      "Always answers 202 with an empty body, whether the address has an account or not. At most one link per address every 60 seconds and five every 24 hours.",
  })
  @ApiAcceptedResponse({ description: "Accepted, with no body" })
  @ApiProblemResponse(HttpStatus.BAD_REQUEST, "VALIDATION_FAILED")
  @ApiProblemResponse(HttpStatus.FORBIDDEN, "INVALID_ORIGIN")
  @ApiProblemResponse(HttpStatus.SERVICE_UNAVAILABLE, REDIS_UNAVAILABLE)
  async requestPasswordReset(@Body() body: EmailDto): Promise<void> {
    await this.passwordResetService.request(body.email);
  }

  @Public()
  @Post("password-resets/confirmation")
  @Throttle({ default: { ttl: ONE_MINUTE, limit: 5 } })
  @HttpCode(HttpStatus.NO_CONTENT)
  @ApiOperation({
    summary: "Set a new password with the emailed token",
    description:
      "Replaces the password, creating the password login for a Google-only account, and drops every session of the user at once.",
  })
  @ApiNoContentResponse({ description: "Password replaced" })
  @ApiProblemResponse(
    HttpStatus.BAD_REQUEST,
    "INVALID_TOKEN, or VALIDATION_FAILED with WEAK_PASSWORD at #/newPassword",
  )
  @ApiProblemResponse(HttpStatus.FORBIDDEN, "INVALID_ORIGIN")
  @ApiProblemResponse(HttpStatus.SERVICE_UNAVAILABLE, REDIS_UNAVAILABLE)
  async confirmPasswordReset(
    @Body() body: PasswordResetConfirmationDto,
  ): Promise<void> {
    await this.passwordResetService.confirm(body.token, body.newPassword);
  }
}
