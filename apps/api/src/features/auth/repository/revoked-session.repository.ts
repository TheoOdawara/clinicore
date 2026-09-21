import { Inject, Injectable } from "@nestjs/common";
import type Redis from "ioredis";
import { BusinessError } from "../../../common/exceptions/business-error";
import { REDIS } from "../../../core/redis/redis";

const KEY_PREFIX = "auth:revoked:";
const TTL_IN_SECONDS = 900;

function unavailable(): BusinessError {
  return BusinessError.unavailable(
    "SERVICE_UNAVAILABLE",
    "Service temporarily unavailable",
  );
}

@Injectable()
export class RevokedSessionRepository {
  constructor(@Inject(REDIS) private readonly redis: Redis) {}

  async revoke(sessionId: string): Promise<void> {
    try {
      await this.redis.set(
        `${KEY_PREFIX}${sessionId}`,
        "1",
        "EX",
        TTL_IN_SECONDS,
      );
    } catch {
      throw unavailable();
    }
  }

  async revokeMany(sessionIds: string[]): Promise<void> {
    for (const sessionId of sessionIds) {
      await this.revoke(sessionId);
    }
  }

  async isRevoked(sessionId: string): Promise<boolean> {
    try {
      const found = await this.redis.exists(`${KEY_PREFIX}${sessionId}`);
      return found === 1;
    } catch {
      throw unavailable();
    }
  }
}
