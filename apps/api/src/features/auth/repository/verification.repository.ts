import { Injectable } from "@nestjs/common";
import { InjectDataSource } from "@nestjs/typeorm";
import { DataSource, In, IsNull, type EntityManager } from "typeorm";
import { Account } from "../entities/account.entity";
import { Session } from "../entities/session.entity";
import { User } from "../entities/user.entity";
import { Verification } from "../entities/verification.entity";
import { Provider } from "../enums/provider.enum";
import { VerificationPurpose } from "../enums/verification-purpose.enum";
import type { IssuedToken } from "../utils/verification-token";

export type EmailConfirmation =
  | { status: "confirmed"; user: User }
  | { status: "invalid" }
  | { status: "expired" };

export type PasswordResetOutcome =
  { status: "reset"; revokedSessionIds: string[] } | { status: "invalid" };

type ConsumedToken =
  | { status: "consumed"; identifier: string }
  | { status: "invalid" }
  | { status: "expired" };

async function consumeToken(
  manager: EntityManager,
  tokenHash: string,
  purpose: VerificationPurpose,
): Promise<ConsumedToken> {
  const verification = await manager.findOne(Verification, {
    where: { tokenHash, purpose },
    lock: { mode: "pessimistic_write" },
  });

  if (verification?.consumedAt !== null) {
    return { status: "invalid" };
  }

  if (verification.expiresAt.getTime() <= Date.now()) {
    return { status: "expired" };
  }

  await manager.update(
    Verification,
    { id: verification.id },
    { consumedAt: new Date() },
  );

  return { status: "consumed", identifier: verification.identifier };
}

export function insertToken(
  manager: EntityManager,
  identifier: string,
  purpose: VerificationPurpose,
  token: IssuedToken,
): Promise<unknown> {
  return manager.insert(Verification, {
    identifier,
    purpose,
    tokenHash: token.tokenHash,
    expiresAt: token.expiresAt,
  });
}

@Injectable()
export class VerificationRepository {
  constructor(@InjectDataSource() private readonly dataSource: DataSource) {}

  async createToken(
    identifier: string,
    purpose: VerificationPurpose,
    token: IssuedToken,
  ): Promise<void> {
    await insertToken(this.dataSource.manager, identifier, purpose, token);
  }

  confirmEmail(tokenHash: string): Promise<EmailConfirmation> {
    return this.dataSource.transaction(async (manager) => {
      const consumed = await consumeToken(
        manager,
        tokenHash,
        VerificationPurpose.EmailVerification,
      );

      if (consumed.status !== "consumed") {
        return consumed;
      }

      const user = await manager.findOne(User, {
        where: { email: consumed.identifier },
      });

      if (user === null) {
        return { status: "invalid" };
      }

      await manager.update(User, { id: user.id }, { emailVerified: true });
      user.emailVerified = true;
      await manager.update(
        Verification,
        {
          identifier: consumed.identifier,
          purpose: VerificationPurpose.EmailVerification,
          consumedAt: IsNull(),
        },
        { consumedAt: new Date() },
      );

      return { status: "confirmed", user };
    });
  }

  resetPassword(
    tokenHash: string,
    passwordHash: string,
  ): Promise<PasswordResetOutcome> {
    return this.dataSource.transaction(async (manager) => {
      const consumed = await consumeToken(
        manager,
        tokenHash,
        VerificationPurpose.PasswordReset,
      );

      if (consumed.status !== "consumed") {
        return { status: "invalid" };
      }

      const user = await manager.findOne(User, {
        where: { email: consumed.identifier },
      });

      if (user === null) {
        return { status: "invalid" };
      }

      const account = await manager.findOne(Account, {
        where: { userId: user.id, provider: Provider.Credential },
      });

      if (account === null) {
        await manager.insert(Account, {
          userId: user.id,
          provider: Provider.Credential,
          passwordHash,
        });
      } else {
        await manager.update(Account, { id: account.id }, { passwordHash });
      }

      const sessions = await manager.find(Session, {
        where: { userId: user.id },
        select: { id: true },
      });
      const revokedSessionIds = sessions.map((session) => session.id);

      if (revokedSessionIds.length > 0) {
        await manager.delete(Session, { id: In(revokedSessionIds) });
      }

      return { status: "reset", revokedSessionIds };
    });
  }
}
