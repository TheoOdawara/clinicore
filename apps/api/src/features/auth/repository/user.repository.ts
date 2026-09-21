import { Injectable } from "@nestjs/common";
import { InjectDataSource } from "@nestjs/typeorm";
import { DataSource, QueryFailedError } from "typeorm";
import { Account } from "../entities/account.entity";
import { User } from "../entities/user.entity";
import { Provider } from "../enums/provider.enum";

const UNIQUE_VIOLATION = "23505";

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

@Injectable()
export class UserRepository {
  constructor(@InjectDataSource() private readonly dataSource: DataSource) {}

  async createWithCredentialAccount(
    name: string,
    email: string,
    passwordHash: string,
  ): Promise<"created" | "duplicate"> {
    try {
      await this.dataSource.transaction(async (manager) => {
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
      });
    } catch (error) {
      if (isUniqueViolation(error)) {
        return "duplicate";
      }
      throw error;
    }

    return "created";
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
