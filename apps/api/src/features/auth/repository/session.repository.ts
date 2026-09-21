import { Injectable } from "@nestjs/common";
import { InjectDataSource } from "@nestjs/typeorm";
import { DataSource, In } from "typeorm";
import { Session } from "../entities/session.entity";
import { RevokedSessionRepository } from "./revoked-session.repository";

export const SESSION_CAP = 5;

export type RotationOutcome =
  | { status: "rotated"; session: Session }
  | { status: "reused" }
  | { status: "invalid" };

@Injectable()
export class SessionRepository {
  constructor(
    @InjectDataSource() private readonly dataSource: DataSource,
    private readonly revoked: RevokedSessionRepository,
  ) {}

  createSession(
    userId: string,
    refreshTokenHash: string,
    expiresAt: Date,
    ipAddress: string | null,
    userAgent: string | null,
  ): Promise<Session> {
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
        await this.revoked.revokeMany(revokedIds);
      }

      return session;
    });
  }

  rotate(
    sessionId: string,
    matchesStoredHash: (storedHash: string) => boolean,
    refreshTokenHash: string,
    expiresAt: Date,
  ): Promise<RotationOutcome> {
    return this.dataSource.transaction(async (manager) => {
      const session = await manager.findOne(Session, {
        where: { id: sessionId },
        lock: { mode: "pessimistic_write" },
      });

      if (session === null || session.expiresAt.getTime() <= Date.now()) {
        return { status: "invalid" };
      }

      if (!matchesStoredHash(session.refreshTokenHash)) {
        await manager.delete(Session, { id: sessionId });
        await this.revoked.revoke(sessionId);
        return { status: "reused" };
      }

      session.refreshTokenHash = refreshTokenHash;
      session.expiresAt = expiresAt;

      return { status: "rotated", session: await manager.save(session) };
    });
  }

  findById(sessionId: string): Promise<Session | null> {
    return this.dataSource
      .getRepository(Session)
      .findOne({ where: { id: sessionId } });
  }

  revokeSession(sessionId: string): Promise<boolean> {
    return this.dataSource.transaction(async (manager) => {
      const result = await manager.delete(Session, { id: sessionId });
      if (result.affected === 0) {
        return false;
      }

      await this.revoked.revoke(sessionId);
      return true;
    });
  }
}
