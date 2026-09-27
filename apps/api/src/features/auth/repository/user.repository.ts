import { Injectable } from "@nestjs/common";
import { InjectDataSource } from "@nestjs/typeorm";
import {
  DataSource,
  IsNull,
  In,
  Not,
  QueryFailedError,
  type EntityManager,
} from "typeorm";
import { Account } from "../entities/account.entity";
import { Session } from "../entities/session.entity";
import { User } from "../entities/user.entity";
import { Verification } from "../entities/verification.entity";
import { EmailDispatchKind } from "../enums/email-dispatch-kind.enum";
import { Provider } from "../enums/provider.enum";
import { VerificationPurpose } from "../enums/verification-purpose.enum";
import {
  claimDispatch,
  isSerializationFailure,
} from "./email-dispatch.repository";
import type { GoogleIdentity } from "../service/google-account.service";
import type { IssuedToken } from "../utils/verification-token";
import { insertToken } from "./verification.repository";

const UNIQUE_VIOLATION = "23505";
const SIGN_UP_ATTEMPTS = 2;
const GOOGLE_LINK_ATTEMPTS = 2;

export type SignUpOutcome = "created-with-token" | "created" | "duplicate";

export interface CredentialAccount {
  user: User;
  account: Account | null;
}

function isUniqueViolation(error: unknown): boolean {
  return (
    error instanceof QueryFailedError &&
    (error.driverError as { code?: string }).code === UNIQUE_VIOLATION
  );
}

async function createAccount(
  manager: EntityManager,
  name: string,
  email: string,
  passwordHash: string,
  token: IssuedToken,
): Promise<SignUpOutcome> {
  const user = await manager.save(
    manager.create(User, { name, email, emailVerified: false }),
  );
  await manager.save(
    manager.create(Account, {
      userId: user.id,
      provider: Provider.Credential,
      passwordHash,
    }),
  );

  if (!(await claimDispatch(manager, email, EmailDispatchKind.Verification))) {
    return "created";
  }

  await insertToken(
    manager,
    email,
    VerificationPurpose.EmailVerification,
    token,
  );

  return "created-with-token";
}

async function verifyAndDropForeignPassword(
  manager: EntityManager,
  user: User,
): Promise<void> {
  await manager.update(User, { id: user.id }, { emailVerified: true });
  user.emailVerified = true;
  await manager.delete(Account, {
    userId: user.id,
    provider: Provider.Credential,
  });
  await manager.update(
    Verification,
    {
      identifier: user.email,
      purpose: VerificationPurpose.EmailVerification,
      consumedAt: IsNull(),
    },
    { consumedAt: new Date() },
  );
}

async function linkGoogleAccount(
  manager: EntityManager,
  identity: GoogleIdentity,
): Promise<User> {
  const linked = await manager.findOne(Account, {
    where: { provider: Provider.Google, providerAccountId: identity.subject },
    relations: { user: true },
  });

  if (linked !== null) {
    return linked.user;
  }

  const existing = await manager.findOne(User, {
    where: { email: identity.email },
  });

  if (existing === null) {
    const created = await manager.save(
      manager.create(User, {
        name: identity.name,
        email: identity.email,
        emailVerified: true,
        image: identity.image,
      }),
    );
    await manager.insert(Account, {
      userId: created.id,
      provider: Provider.Google,
      providerAccountId: identity.subject,
    });

    return created;
  }

  if (!existing.emailVerified) {
    await verifyAndDropForeignPassword(manager, existing);
  }

  await manager.insert(Account, {
    userId: existing.id,
    provider: Provider.Google,
    providerAccountId: identity.subject,
  });

  return existing;
}

@Injectable()
export class UserRepository {
  constructor(@InjectDataSource() private readonly dataSource: DataSource) {}

  async createWithCredentialAccount(
    name: string,
    email: string,
    passwordHash: string,
    token: IssuedToken,
  ): Promise<SignUpOutcome> {
    for (let attempt = 1; ; attempt += 1) {
      try {
        return await this.dataSource.transaction("SERIALIZABLE", (manager) =>
          createAccount(manager, name, email, passwordHash, token),
        );
      } catch (error) {
        if (isUniqueViolation(error)) {
          return "duplicate";
        }
        if (isSerializationFailure(error) && attempt < SIGN_UP_ATTEMPTS) {
          continue;
        }
        throw error;
      }
    }
  }

  async linkOrCreateGoogleAccount(identity: GoogleIdentity): Promise<User> {
    for (let attempt = 1; ; attempt += 1) {
      try {
        return await this.dataSource.transaction("SERIALIZABLE", (manager) =>
          linkGoogleAccount(manager, identity),
        );
      } catch (error) {
        const concurrent =
          isUniqueViolation(error) || isSerializationFailure(error);
        if (concurrent && attempt < GOOGLE_LINK_ATTEMPTS) {
          continue;
        }
        throw error;
      }
    }
  }

  findByEmail(email: string): Promise<User | null> {
    return this.dataSource.getRepository(User).findOne({ where: { email } });
  }

  async findByEmailWithCredentialAccount(
    email: string,
  ): Promise<CredentialAccount | null> {
    const user = await this.dataSource
      .getRepository(User)
      .findOne({ where: { email }, relations: { accounts: true } });

    if (user === null) {
      return null;
    }

    const account = user.accounts.find(
      (candidate) => candidate.provider === Provider.Credential,
    );

    return { user, account: account ?? null };
  }

  findById(id: string): Promise<User | null> {
    return this.dataSource.getRepository(User).findOne({ where: { id } });
  }

  async findCredentialHash(userId: string): Promise<string | null> {
    const account = await this.dataSource.getRepository(Account).findOne({
      where: { userId, provider: Provider.Credential },
      select: { passwordHash: true },
    });

    return account?.passwordHash ?? null;
  }

  changePassword(
    userId: string,
    keptSessionId: string,
    currentHash: string,
    newHash: string,
  ): Promise<
    { status: "changed"; revokedSessionIds: string[] } | { status: "invalid" }
  > {
    return this.dataSource.transaction(async (manager) => {
      const updated = await manager.update(
        Account,
        {
          userId,
          provider: Provider.Credential,
          passwordHash: currentHash,
        },
        { passwordHash: newHash },
      );

      if (updated.affected === 0) {
        return { status: "invalid" };
      }

      const sessions = await manager.find(Session, {
        where: { userId, id: Not(keptSessionId) },
        select: { id: true },
      });
      const revokedSessionIds = sessions.map((session) => session.id);

      if (revokedSessionIds.length > 0) {
        await manager.delete(Session, { id: In(revokedSessionIds) });
      }

      return { status: "changed", revokedSessionIds };
    });
  }
}
