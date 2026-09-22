import { Injectable } from "@nestjs/common";
import { InjectDataSource } from "@nestjs/typeorm";
import { DataSource, QueryFailedError, type EntityManager } from "typeorm";
import { Account } from "../entities/account.entity";
import { User } from "../entities/user.entity";
import { EmailDispatchKind } from "../enums/email-dispatch-kind.enum";
import { Provider } from "../enums/provider.enum";
import { VerificationPurpose } from "../enums/verification-purpose.enum";
import {
  claimDispatch,
  isSerializationFailure,
} from "./email-dispatch.repository";
import type { IssuedToken } from "../utils/verification-token";
import { insertToken } from "./verification.repository";

const UNIQUE_VIOLATION = "23505";
const SIGN_UP_ATTEMPTS = 2;

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
}
