import { Module } from "@nestjs/common";
import { JwtModule } from "@nestjs/jwt";
import { PassportModule } from "@nestjs/passport";
import { TypeOrmModule } from "@nestjs/typeorm";
import { EnvironmentService } from "../../core/config/environment.service";
import { AuthController } from "./controller/auth.controller";
import { Account } from "./entities/account.entity";
import { EmailDispatch } from "./entities/email-dispatch.entity";
import { Session } from "./entities/session.entity";
import { User } from "./entities/user.entity";
import { Verification } from "./entities/verification.entity";
import { EmailDispatchRepository } from "./repository/email-dispatch.repository";
import { RevokedSessionRepository } from "./repository/revoked-session.repository";
import { SessionRepository } from "./repository/session.repository";
import { UserRepository } from "./repository/user.repository";
import { VerificationRepository } from "./repository/verification.repository";
import {
  DUMMY_PASSWORD_HASH,
  createDummyPasswordHash,
} from "./service/dummy-password-hash";
import { EmailVerificationService } from "./service/email-verification.service";
import { OpenSessionService } from "./service/open-session.service";
import { PasswordResetService } from "./service/password-reset.service";
import { RefreshSessionService } from "./service/refresh-session.service";
import { SessionService } from "./service/session.service";
import { SignInService } from "./service/sign-in.service";
import { SignOutService } from "./service/sign-out.service";
import { SignUpService } from "./service/sign-up.service";
import { JwtStrategy } from "./strategy/jwt.strategy";
import { ACCESS_MAX_AGE_IN_SECONDS } from "./utils/session-cookies";

@Module({
  imports: [
    TypeOrmModule.forFeature([
      User,
      Account,
      Session,
      Verification,
      EmailDispatch,
    ]),
    PassportModule,
    JwtModule.registerAsync({
      inject: [EnvironmentService],
      useFactory: (environment: EnvironmentService) => ({
        secret: environment.get("JWT_SECRET"),
        signOptions: { expiresIn: ACCESS_MAX_AGE_IN_SECONDS },
      }),
    }),
  ],
  controllers: [AuthController],
  providers: [
    UserRepository,
    SessionRepository,
    RevokedSessionRepository,
    EmailDispatchRepository,
    VerificationRepository,
    OpenSessionService,
    EmailVerificationService,
    PasswordResetService,
    SignUpService,
    SignInService,
    RefreshSessionService,
    SignOutService,
    SessionService,
    JwtStrategy,
    { provide: DUMMY_PASSWORD_HASH, useFactory: createDummyPasswordHash },
  ],
})
export class AuthModule {}
