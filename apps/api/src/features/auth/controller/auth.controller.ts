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
import { EnvironmentService } from "../../../core/config/environment.service";
import { SessionResponse } from "../dto/session.response";
import { SignInDto } from "../dto/sign-in.dto";
import { SignUpDto } from "../dto/sign-up.dto";
import { RefreshSessionService } from "../service/refresh-session.service";
import { SessionService } from "../service/session.service";
import { SignInService } from "../service/sign-in.service";
import { SignOutService } from "../service/sign-out.service";
import { SignUpService } from "../service/sign-up.service";
import {
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

@ApiTags("Auth")
@ApiProblemResponse(HttpStatus.TOO_MANY_REQUESTS, "RATE_LIMITED")
@Controller()
export class AuthController {
  constructor(
    private readonly signUpService: SignUpService,
    private readonly signInService: SignInService,
    private readonly refreshSessionService: RefreshSessionService,
    private readonly signOutService: SignOutService,
    private readonly sessionService: SessionService,
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
  @ApiOperation({ summary: "Open a session and issue the session cookies" })
  @ApiCreatedResponse({
    description: "The signed-in user",
    type: SessionResponse,
    headers: {
      ...SESSION_COOKIES,
      Location: {
        description: "The session that was opened",
        schema: { type: "string", example: CURRENT_SESSION_PATH },
      },
    },
  })
  @ApiProblemResponse(HttpStatus.BAD_REQUEST, "VALIDATION_FAILED")
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
    @Req() request: Request,
    @Res({ passthrough: true }) response: Response,
  ): Promise<SessionResponse> {
    const opened = await this.signInService.signIn(
      body.email,
      body.password,
      request.ip ?? null,
      userAgentOf(request),
    );

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
      "Authenticated by the clinicore_refresh cookie. Presenting a token that was already rotated drops the whole session.",
  })
  @ApiNoContentResponse({
    description: "Rotated, with both cookies replaced",
    headers: SESSION_COOKIES,
  })
  @ApiProblemResponse(
    HttpStatus.UNAUTHORIZED,
    "INVALID_SESSION, or SESSION_REUSED when the old token comes back",
  )
  @ApiProblemResponse(HttpStatus.FORBIDDEN, "INVALID_ORIGIN")
  @ApiProblemResponse(HttpStatus.SERVICE_UNAVAILABLE, REDIS_UNAVAILABLE)
  async refresh(
    @Req() request: Request,
    @Res({ passthrough: true }) response: Response,
  ): Promise<void> {
    const rotated = await this.refreshSessionService.refresh(
      refreshTokenOf(request),
    );

    setSessionCookies(response, this.environment.get("NODE_ENV"), rotated);
  }

  @Delete("sessions/current")
  @HttpCode(HttpStatus.NO_CONTENT)
  @ApiOperation({
    summary: "Close the current session",
    description:
      "Deletes the session row and denies the access token that is still inside its 15 minutes.",
  })
  @ApiNoContentResponse({
    description: "Signed out, with both cookies expired",
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
    @Res({ passthrough: true }) response: Response,
  ): Promise<void> {
    await this.signOutService.signOut(user.sessionId);

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
}
