import { Injectable } from "@nestjs/common";
import { InjectDataSource } from "@nestjs/typeorm";
import { DataSource, In } from "typeorm";
import { Session } from "../entities/session.entity";

export const SESSION_CAP = 5;

export interface CreatedSession {
  session: Session;
  revokedIds: string[];
}

@Injectable()
export class SessionRepository {
  constructor(@InjectDataSource() private readonly dataSource: DataSource) {}

  createSession(
    userId: string,
    refreshTokenHash: string,
    expiresAt: Date,
    ipAddress: string | null,
    userAgent: string | null,
  ): Promise<CreatedSession> {
    return this.dataSource.transaction(async (manager) => {
      const session = await manager.save(
        manager.create(Session, {
          userId,
          refreshTokenHash,
          expiresAt,
          ipAddress,
          userAgent,
        }),
      );

      const surplus = await manager.find(Session, {
        where: { userId },
        order: { createdAt: "DESC", id: "DESC" },
        skip: SESSION_CAP,
        select: { id: true },
      });

      const revokedIds = surplus.map((expired) => expired.id);
      if (revokedIds.length > 0) {
        await manager.delete(Session, { id: In(revokedIds) });
      }

      return { session, revokedIds };
    });
  }

  findById(sessionId: string): Promise<Session | null> {
    return this.dataSource
      .getRepository(Session)
      .findOne({ where: { id: sessionId } });
  }
}
