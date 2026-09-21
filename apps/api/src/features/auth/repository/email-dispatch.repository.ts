import { Injectable } from "@nestjs/common";
import { InjectDataSource } from "@nestjs/typeorm";
import {
  DataSource,
  MoreThan,
  QueryFailedError,
  type EntityManager,
} from "typeorm";
import { EmailDispatch } from "../entities/email-dispatch.entity";
import { EmailDispatchKind } from "../enums/email-dispatch-kind.enum";

const REPEAT_WINDOW_IN_MILLISECONDS = 60 * 1000;
const DAILY_WINDOW_IN_MILLISECONDS = 24 * 60 * 60 * 1000;
const DAILY_CAP = 5;
const SERIALIZATION_FAILURE = "40001";

export function isSerializationFailure(error: unknown): boolean {
  return (
    error instanceof QueryFailedError &&
    (error.driverError as { code?: string }).code === SERIALIZATION_FAILURE
  );
}

export async function claimDispatch(
  manager: EntityManager,
  email: string,
  kind: EmailDispatchKind,
): Promise<boolean> {
  const now = Date.now();
  const recent = await manager.find(EmailDispatch, {
    where: {
      email,
      kind,
      createdAt: MoreThan(new Date(now - DAILY_WINDOW_IN_MILLISECONDS)),
    },
    order: { createdAt: "DESC" },
    select: { createdAt: true },
  });

  if (recent.length >= DAILY_CAP) {
    return false;
  }

  const latest = recent[0];
  if (
    latest !== undefined &&
    latest.createdAt.getTime() > now - REPEAT_WINDOW_IN_MILLISECONDS
  ) {
    return false;
  }

  await manager.insert(EmailDispatch, { email, kind });

  return true;
}

@Injectable()
export class EmailDispatchRepository {
  constructor(@InjectDataSource() private readonly dataSource: DataSource) {}

  async registerDispatch(
    email: string,
    kind: EmailDispatchKind,
  ): Promise<boolean> {
    try {
      return await this.dataSource.transaction("SERIALIZABLE", (manager) =>
        claimDispatch(manager, email, kind),
      );
    } catch (error) {
      if (isSerializationFailure(error)) {
        return false;
      }
      throw error;
    }
  }
}
